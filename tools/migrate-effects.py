#!/usr/bin/env python3
"""One-shot migration: convert `[[actions.effects]]` / `[[npc_actions.effects]]`
TOML table-arrays into a single `effect = '<rhai call-list>'` field per action.

Run once from the repo root. validate-pack is the gate for correctness.
"""
import re
import glob

HDR = re.compile(r"^(\s*)\[\[(actions|npc_actions)\.effects\]\]\s*$")
FIELD = re.compile(r"^\s*([A-Za-z_]\w*)\s*=\s*(.+?)\s*$")


def parse_val(v):
    v = v.strip()
    if len(v) >= 2 and v[0] == '"' and v[-1] == '"':
        return ("str", v[1:-1])
    if v in ("true", "false"):
        return ("bool", v)
    if re.fullmatch(r"-?\d+", v):
        return ("int", v)
    return ("raw", v)


def q(x):
    return '"%s"' % x


def call_for(f):
    """f: dict name -> (kind, text); 'type' -> ('str', name)."""
    t = f["type"][1]

    def sv(k):
        return f[k][1]

    simple = {
        "change_stress": lambda: f"w.changeStress({sv('amount')})",
        "change_money": lambda: f"w.changeMoney({sv('amount')})",
        "change_anxiety": lambda: f"w.changeAnxiety({sv('amount')})",
        "add_arousal": lambda: f"w.addArousal({sv('delta')})",
        "change_alcohol": lambda: f"w.changeAlcohol({sv('delta')})",
        "skill_increase": lambda: f"w.skillIncrease({q(sv('skill'))}, {sv('amount')})",
        "add_trait": lambda: f"w.addTrait({q(sv('trait_id'))})",
        "remove_trait": lambda: f"w.removeTrait({q(sv('trait_id'))})",
        "add_stuff": lambda: f"w.addStuff({q(sv('item'))})",
        "remove_stuff": lambda: f"w.removeStuff({q(sv('item'))})",
        "set_player_partner": lambda: f"w.setPartner({q(sv('npc'))})",
        "add_player_friend": lambda: f"w.addFriend({q(sv('npc'))})",
        "set_scene_flag": lambda: f"scene.setFlag({q(sv('flag'))})",
        "remove_scene_flag": lambda: f"scene.removeFlag({q(sv('flag'))})",
        "set_game_flag": lambda: f"gd.setGameFlag({q(sv('flag'))})",
        "remove_game_flag": lambda: f"gd.removeGameFlag({q(sv('flag'))})",
        "add_stat": lambda: f"gd.addStat({q(sv('stat'))}, {sv('amount')})",
        "set_stat": lambda: f"gd.setStat({q(sv('stat'))}, {sv('value')})",
        "set_job_title": lambda: f"gd.setJobTitle({q(sv('title'))})",
        "advance_time": lambda: f"gd.advanceTime({sv('slots')})",
        "advance_arc": lambda: f"gd.advanceArc({q(sv('arc'))}, {q(sv('to_state'))})",
        "fail_red_check": lambda: f"gd.failRedCheck({q(sv('skill'))})",
        "add_npc_liking": lambda: f"npc({q(sv('npc'))}).addLiking({sv('delta')})",
        "add_npc_love": lambda: f"npc({q(sv('npc'))}).addLove({sv('delta')})",
        "add_w_liking": lambda: f"npc({q(sv('npc'))}).addWLiking({sv('delta')})",
        "set_npc_flag": lambda: f"npc({q(sv('npc'))}).setFlag({q(sv('flag'))})",
        "add_npc_trait": lambda: f"npc({q(sv('npc'))}).addTrait({q(sv('trait_id'))})",
        "set_relationship": lambda: f"npc({q(sv('npc'))}).setRelationship({q(sv('status'))})",
        "set_npc_attraction": lambda: f"npc({q(sv('npc'))}).setAttraction({sv('delta')})",
        "set_npc_behaviour": lambda: f"npc({q(sv('npc'))}).setBehaviour({q(sv('behaviour'))})",
        "set_contactable": lambda: f"npc({q(sv('npc'))}).setContactable({sv('value')})",
        "add_sexual_activity": lambda: f"npc({q(sv('npc'))}).addSexualActivity({q(sv('activity'))})",
        "set_npc_role": lambda: f"npc({q(sv('npc'))}).setRole({q(sv('role'))})",
        "set_npc_name": lambda: f"npc({q(sv('npc'))}).setName({q(sv('name'))})",
    }
    if t == "set_virgin":
        if "virgin_type" in f:
            return f"w.setVirgin({sv('value')}, {q(sv('virgin_type'))})"
        return f"w.setVirgin({sv('value')})"
    if t == "transition":
        return None  # dead no-op in the legacy engine; drop
    if t not in simple:
        raise SystemExit(f"UNKNOWN effect type: {t}")
    return simple[t]()


def migrate_text(lines):
    out = []
    i = 0
    n = len(lines)
    while i < n:
        m = HDR.match(lines[i])
        if not m:
            out.append(lines[i])
            i += 1
            continue
        indent = m.group(1)
        kind = m.group(2)
        calls = []
        # consume a run of same-kind effect tables (blank lines between allowed)
        while i < n:
            # skip blank lines only if a same-kind effect header follows
            if lines[i].strip() == "":
                j = i
                while j < n and lines[j].strip() == "":
                    j += 1
                hm = HDR.match(lines[j]) if j < n else None
                if hm and hm.group(2) == kind:
                    i = j
                    continue
                break
            hm = HDR.match(lines[i])
            if not hm or hm.group(2) != kind:
                break
            i += 1  # past the header
            fields = {}
            while i < n:
                if lines[i].strip() == "":
                    break
                fm = FIELD.match(lines[i])
                if not fm:
                    break
                fields[fm.group(1)] = parse_val(fm.group(2))
                i += 1
            c = call_for(fields)
            if c is not None:
                calls.append(c)
        if calls:
            out.append(f"{indent}effect = '{'; '.join(calls)};'")
        # if only transition(s) -> emit nothing
    return out


def main():
    changed = 0
    for fp in sorted(glob.glob("packs/base/scenes/*.toml")):
        orig = open(fp, encoding="utf-8").read()
        if ".effects]]" not in orig:
            continue
        lines = orig.split("\n")
        new = "\n".join(migrate_text(lines))
        if new != orig:
            open(fp, "w", encoding="utf-8", newline="").write(new)
            changed += 1
    print(f"migrated effects in {changed} scene files")


if __name__ == "__main__":
    main()
