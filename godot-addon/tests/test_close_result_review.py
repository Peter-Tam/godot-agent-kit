#!/usr/bin/env python3
"""Success claims must carry their own acquired closing postconditions."""
import copy
import unittest

import run_observation as observation
from close_result_review import review_close_result


def verified_result(unloaded=False):
    target = {"project_root": "/owned/project", "session_id": "a" * 32,
              "script_path": "res://scripts/subject.gd"}
    def stamp(authority="R", tick=20):
        return {"clock_id": "caller" if authority == "D" else "editor:" + target["session_id"],
                "started_tick_us": str(tick), "finished_tick_us": str(tick), "received_elapsed_us": tick}
    def source(authority, tick=20):
        return {"authority": authority, "availability": "observed",
                "digest": {"sha256": "b" * 64, "utf8_bytes": 12}, "collection": stamp(authority, tick),
                "witness": {"resource_path": target["script_path"], "script_instance_id": "11" if authority == "R" else None,
                            "disk_file_id": {"device": "1", "inode": "2"} if authority == "D" else None},
                "staleness": {"state": "unknown"}, "reason": None, "invalidated_evidence": None}
    def fact(value, authority="R"):
        return {"value": value, "collection": stamp(authority), "reason": None, "invalidated_evidence": None}
    resource = source("R") if not unloaded else {
        "authority": "R", "availability": "unavailable", "digest": None,
        "reason": {"code": "resource_not_loaded"}, "invalidated_evidence": None}
    result = {"schema_version": 1, "operation": "close_gdscript", "request_id": "c" * 32,
              "requested_target": target, "resolved_target": target,
              "interval": {"started_unix_ms": 100, "finished_unix_ms": 101, "elapsed_us": 40},
              "outcome": "verified_newly_closed", "reason": "complete", "stage": "terminal", "application": "applied",
              "expected": None,
              "progress": {name: {"state": "completed", "reason": None, "collection": stamp(tick=10)}
                           for name in ("closing", "native_revalidation", "verification")},
              "before": {"document": {"identity": {"script_instance_id": "11"}},
                         "sources": {"D": source("D", 5), "R": source("R", 5)}},
              "observation": {"purpose": "verification",
                  "interval": {"started_unix_ms": 100, "finished_unix_ms": 101, "elapsed_us": 30},
                  "snapshot": {"target": target,
                      "document": {"identity": {"kind": "external_gdscript", "resource_path": target["script_path"],
                                                "script_instance_id": None if unloaded else "11",
                                                "editor_instance_id": None, "buffer_instance_id": None},
                                   "open_state": fact("not_open"), "validity": fact("valid", "D" if unloaded else "R")},
                      "sources": {"D": source("D"), "R": resource,
                                  "B": {"authority": "B", "availability": "not_applicable", "digest": None,
                                        "invalidated_evidence": None}},
                      "dirty": {"availability": "not_applicable", "state": "not_applicable", "invalidated_evidence": None},
                      "consistency": {"checks": "performed", "detected_changes": [], "recheck_reason": None,
                                      "stability": "unknown", "atomic": False},
                      "comparisons": {"disk_resource": "equal" if not unloaded else "unknown"}}},
              "resource_state": {"state": "unloaded" if unloaded else "retained", "resource_edited": None if unloaded else False,
                                 "reason": None, "collection": stamp()},
              "protection": {"status": "preserved", "revalidation": "completed", "required_count": 1,
                             "completed_count": 1, "reason": None},
              "selection": None, "history": {"participation": "not_participated", "target_buffer": "disposed",
                                              "unrelated": "observed", "reason": None},
              "diagnostics": [],
              "safe_next_action": "Obtain a fresh ordinary observation of the explicit original target before another intentional action"}
    return result


class CloseResultReviewTests(unittest.TestCase):
    def test_retained_and_unloaded_are_distinct_valid_postconditions(self):
        for unloaded in (False, True):
            with self.subTest(unloaded=unloaded):
                result = verified_result(unloaded)
                self.assertEqual(review_close_result(result)["resource"], "unloaded" if unloaded else "retained")

    def test_incomplete_or_invalidated_postconditions_cannot_certify_closure(self):
        changes = [
            (("before",), None),
            (("observation", "snapshot", "sources", "D", "availability"), "unavailable"),
            (("observation", "snapshot", "sources", "D", "witness"), None),
            (("observation", "snapshot", "sources", "D", "digest", "sha256"), "d" * 64),
            (("observation", "snapshot", "sources", "D", "collection", "received_elapsed_us"), 5),
            (("observation", "snapshot", "sources", "R", "availability"), "unavailable"),
            (("observation", "snapshot", "sources", "R", "witness", "script_instance_id"), "99"),
            (("observation", "snapshot", "sources", "B", "availability"), "unavailable"),
            (("observation", "snapshot", "document", "validity", "value"), "missing"),
            (("observation", "snapshot", "document", "open_state", "value"), "open"),
            (("observation", "snapshot", "document", "open_state", "collection", "clock_id"), "caller"),
            (("observation", "snapshot", "consistency", "checks"), "unavailable"),
            (("observation", "snapshot", "consistency", "detected_changes"), [{"surface": "D", "code": "source_changed"}]),
            (("observation", "snapshot", "dirty", "invalidated_evidence"), {"state": "clean"}),
            (("observation", "snapshot", "sources", "D", "invalidated_evidence"), {"digest": {"sha256": "e" * 64}}),
            (("observation", "snapshot", "sources", "D", "staleness"), {}),
            (("protection", "completed_count"), 0),
            (("progress", "native_revalidation", "state"), "entered"),
            (("safe_next_action",), "Retry closing"),
        ]
        for path, value in changes:
            with self.subTest(path=path):
                result = copy.deepcopy(verified_result())
                field = result
                for key in path[:-1]: field = field[key]
                field[path[-1]] = value
                with self.assertRaises(observation.Failure):
                    review_close_result(result)

    def test_unreadable_resource_is_not_evidence_of_unloading(self):
        result = verified_result(unloaded=True)
        result["observation"]["snapshot"]["sources"]["R"]["reason"]["code"] = "resource_unreadable"
        with self.assertRaises(observation.Failure):
            review_close_result(result)

    def test_missing_authority_cannot_certify_closure(self):
        for authority in ("D", "R", "B"):
            with self.subTest(authority=authority):
                result = verified_result()
                del result["observation"]["snapshot"]["sources"][authority]
                with self.assertRaises(observation.Failure):
                    review_close_result(result)

    def test_caller_validity_requires_its_actual_independent_disk_collection(self):
        result = verified_result(unloaded=True)
        result["observation"]["snapshot"]["document"]["validity"]["collection"]["started_tick_us"] = "19"
        with self.assertRaises(observation.Failure):
            review_close_result(result)


if __name__ == "__main__":
    unittest.main()
