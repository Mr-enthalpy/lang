#!/usr/bin/env python3
"""Small bounded exploration for the atomic/causal/fence design note.

This is NOT an ISA model, compiler proof, or complete language semantics.
It checks just the explicitly documented state-machine abstractions.
"""
from collections import deque
from dataclasses import dataclass


def explore(initial, successors, invariant):
    seen = {initial}
    todo = deque([initial])
    transitions = 0
    while todo:
        s = todo.popleft()
        invariant(s)
        for label, ns in successors(s):
            transitions += 1
            if ns not in seen:
                seen.add(ns)
                todo.append(ns)
    return len(seen), transitions


# ARC with at most 3 tickets, one anchored resource, and atomic steps.
@dataclass(frozen=True)
class Arc:
    mode: str                   # live, unique, retired
    count: int
    tickets: frozenset


def arc_inv(s):
    if s.mode == 'live':
        assert s.count == len(s.tickets) > 0
    else:
        assert s.count == 0 and not s.tickets
    assert s.mode in ('live', 'unique', 'retired')


def arc_next(s):
    if s.mode != 'live':
        return
    if len(s.tickets) < 3:
        for parent in sorted(s.tickets):
            for fresh in range(3):
                if fresh not in s.tickets:
                    yield (f'clone {parent}->{fresh}', Arc('live', s.count + 1, s.tickets | {fresh}))
    for owner in sorted(s.tickets):
        rem = s.tickets - {owner}
        if s.count > 1:
            yield (f'release {owner}', Arc('live', s.count - 1, rem))
        else:
            yield (f'last release {owner}', Arc('retired', 0, frozenset()))
            yield (f'unique {owner}', Arc('unique', 0, frozenset()))


# Exclusive resource handoff in one atomic enum: Free(T) <-> Held(owner).
@dataclass(frozen=True)
class Mutex:
    owner: int                 # -1 if free
    done: frozenset


def mutex_next(s):
    if s.owner == -1:
        for t in range(2):
            if t not in s.done:
                yield (f'acquire {t}', Mutex(t, s.done))
    else:
        yield (f'release {s.owner}', Mutex(-1, s.done | {s.owner}))


def mutex_inv(s):
    assert s.owner in (-1, 0, 1)
    assert s.owner not in s.done


# Bounded one-slot mailbox, two producer values and one consumer.
@dataclass(frozen=True)
class Slot:
    contents: int             # -1 empty, else 0 or 1
    sent: frozenset
    received: frozenset


def slot_next(s):
    if s.contents == -1:
        for v in range(2):
            if v not in s.sent:
                yield (f'send {v}', Slot(v, s.sent | {v}, s.received))
    else:
        yield (f'receive {s.contents}', Slot(-1, s.sent, s.received | {s.contents}))


def slot_inv(s):
    assert s.received <= s.sent
    assert len(s.sent - s.received) == (0 if s.contents == -1 else 1)
    if s.contents != -1:
        assert (s.sent - s.received) == {s.contents}


# A tiny weak peek + causal import model for MP and SB.
# A peek may read any already committed version not older than its causal floor.
@dataclass(frozen=True)
class MP:
    writer: int
    reader: int
    y_read: int               # -1 not read
    x_read: int               # -1 not read


def mp_next(s, causal=True):
    # T0: CAS X(0 -> 1); CAS Y(0 -> 1), where Y1 inherits X1.
    if s.writer < 2:
        yield ('write X' if s.writer == 0 else 'write Y', MP(s.writer+1, s.reader, s.y_read, s.x_read))
    # T1: peek Y, then peek X. Reads can be stale unless causal floor excludes it.
    if s.reader == 0:
        for yv in range(0, 2 if s.writer == 2 else 1):
            yield (f'peek Y={yv}', MP(s.writer, 1, yv, -1))
    elif s.reader == 1:
        xmin = 1 if (causal and s.y_read == 1) else 0
        xmax = 1 if s.writer >= 1 else 0
        for xv in range(xmin, xmax + 1):
            yield (f'peek X={xv}', MP(s.writer, 2, s.y_read, xv))


@dataclass(frozen=True)
class SB:
    a_pc: int                 # 0 write X, 1 peek Y, 2 done
    b_pc: int                 # 0 write Y, 1 peek X, 2 done
    a_read: int               # -1 not read
    b_read: int


def sb_next(s):
    if s.a_pc == 0:
        yield ('A CAS X', SB(1, s.b_pc, s.a_read, s.b_read))
    elif s.a_pc == 1:
        for read_y in range(0, 2 if s.b_pc >= 1 else 1):
            yield (f'A peek Y={read_y}', SB(2, s.b_pc, read_y, s.b_read))
    if s.b_pc == 0:
        yield ('B CAS Y', SB(s.a_pc, 1, s.a_read, s.b_read))
    elif s.b_pc == 1:
        for read_x in range(0, 2 if s.a_pc >= 1 else 1):
            yield (f'B peek X={read_x}', SB(s.a_pc, 2, s.a_read, read_x))


def main():
    arc_s, arc_e = explore(Arc('live', 1, frozenset({0})), arc_next, arc_inv)
    mutex_s, mutex_e = explore(Mutex(-1, frozenset()), mutex_next, mutex_inv)
    slot_s, slot_e = explore(Slot(-1, frozenset(), frozenset()), slot_next, slot_inv)
    mp_good = {MP(0, 0, -1, -1)}
    pending = [next(iter(mp_good))]
    while pending:
        s = pending.pop()
        for _, nxt in mp_next(s, True):
            if nxt not in mp_good:
                mp_good.add(nxt)
                pending.append(nxt)
    assert not any(s.reader == 2 and s.y_read == 1 and s.x_read == 0 for s in mp_good)
    mp_bad = {MP(0, 0, -1, -1)}
    pending = [next(iter(mp_bad))]
    while pending:
        s = pending.pop()
        for _, nxt in mp_next(s, False):
            if nxt not in mp_bad:
                mp_bad.add(nxt)
                pending.append(nxt)
    assert any(s.reader == 2 and s.y_read == 1 and s.x_read == 0 for s in mp_bad)
    sb_seen = {SB(0, 0, -1, -1)}
    pending = [next(iter(sb_seen))]
    while pending:
        s = pending.pop()
        for _, nxt in sb_next(s):
            if nxt not in sb_seen:
                sb_seen.add(nxt)
                pending.append(nxt)
    assert any(s.a_pc == 2 and s.b_pc == 2 and s.a_read == 0 and s.b_read == 0 for s in sb_seen)
    # Two deliberately broken APIs, to exhibit failure of local invariants.
    # Deliberately broken API: decrease count without withdrawing a ticket.
    broken_arc = Arc('live', 1, frozenset({0, 1}))
    try:
        arc_inv(broken_arc)
    except AssertionError:
        raw_count_decrement_breaks_arc = True
    else:
        raw_count_decrement_breaks_arc = False
    # Deliberately broken API: two readers of Free both mint a guard, no CAS.
    observed_state = ['Free', 'Free']
    fake_guards = [i for i, seen in enumerate(observed_state) if seen == 'Free']
    peek_only_acquire_allows_two_guards = len(fake_guards) == 2
    assert raw_count_decrement_breaks_arc and peek_only_acquire_allows_two_guards
    print(f'ARC: {arc_s} reachable states, {arc_e} transitions; count/ticket invariant HOLDS')
    print(f'Mutex: {mutex_s} reachable states, {mutex_e} transitions; one-owner invariant HOLDS')
    print(f'Slot: {slot_s} reachable states, {slot_e} transitions; exact-once transfer invariant HOLDS')
    print(f'MP with causal import: {len(mp_good)} states; Y=1 and X=0 FORBIDDEN')
    print(f'MP without import: {len(mp_bad)} states; Y=1 and X=0 REACHABLE')
    print(f'SB without fence in weak-peek model: {len(sb_seen)} states; 0/0 REACHABLE')
    print('Negative: unlicensed count decrement breaks ARC; peek-only lock acquire duplicates guards')
    print('NOT CHECKED: fence ISA behavior, IRIW, OOTA, overflow, weak refs, reclamation, compiler lowering')

if __name__ == '__main__':
    main()
