use super::*;

const DIFF: &str = "\
diff --git a/features/record.feature b/features/record.feature
--- a/features/record.feature
+++ b/features/record.feature
@@ -1,3 +1,3 @@
-  @REC-01 @P0 @pending
+  @REC-01 @P0
   Scenario: REC-02 stays as it is
diff --git a/docs/old.md b/docs/new.md
similarity index 90%
rename from docs/old.md
rename to docs/new.md
";

#[test]
fn changed_paths_reads_both_sides_of_every_header() {
    assert_eq!(
        changed_paths(DIFF),
        ["docs/new.md", "docs/old.md", "features/record.feature"]
    );
    assert!(changed_paths("not a diff").is_empty());
    assert!(changed_paths("diff --git a/only-one-side").is_empty());
}

#[test]
fn mentioned_ids_reads_only_changed_lines() {
    assert_eq!(mentioned_ids(DIFF), ["REC-01"]);
    assert_eq!(
        mentioned_ids("+++ b/ENG-21\n--- a/ENG-22\n+ see ENG-28, and NOT-01 or ENG-2\n"),
        ["ENG-28"]
    );
}
