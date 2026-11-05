use std::path::PathBuf;
use tuhc_lib::data::ArchiveStore;

#[tokio::test]
async fn test_non_numeric_and_boundary_keys() {
    let asset_dir = std::env::var("TUHC_ASSET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/criper/Downloads/Asset Pack V2"));
    if !asset_dir.exists() {
        eprintln!(
            "Skipping test_non_numeric_and_boundary_keys: asset directory {:?} does not exist",
            asset_dir
        );
        return;
    }
    let store = ArchiveStore::load_from_dir(&asset_dir);

    // 1. Empirically verify specific non-numeric keys in story store
    let pony = store
        .get_story_page("pony")
        .await
        .expect("Key 'pony' must be found in story store");
    assert_eq!(pony.page_id, "pony");
    assert_eq!(pony.title, "[S] Ride.");
    assert_eq!(pony.media, vec!["/storyfiles/hs2/pony/pony.swf"]);

    let darkcage = store
        .get_story_page("darkcage")
        .await
        .expect("Key 'darkcage' must be found in story store");
    assert_eq!(darkcage.page_id, "darkcage");
    assert_eq!(darkcage.title, "[S] Cage: Reveal plan.");
    assert_eq!(
        darkcage.media,
        vec!["/storyfiles/hs2/darkcage/darkcage.swf"]
    );

    let jb2 = store
        .get_story_page("jb2_000000")
        .await
        .expect("Key 'jb2_000000' must be found in story store");
    assert_eq!(jb2.page_id, "jb2_000000");
    assert_eq!(jb2.title, "Enjoy restful slumber.");
    assert_eq!(jb2.media, vec!["/storyfiles/jb2/YOUWIN.gif"]);
    assert_eq!(jb2.previous.as_deref(), Some("000135"));

    let mc = store
        .get_story_page("mc0001")
        .await
        .expect("Key 'mc0001' must be found in story store");
    assert_eq!(mc.page_id, "mc0001");
    assert_eq!(mc.title, "??????");
    assert_eq!(mc.media, vec!["/storyfiles/mc/00000.gif"]);

    // Check secondary non-numeric keys
    let pony2 = store.get_story_page("pony2").await.expect("pony2");
    assert_eq!(pony2.page_id, "pony2");

    let darkcage2 = store.get_story_page("darkcage2").await.expect("darkcage2");
    assert_eq!(darkcage2.page_id, "darkcage2");

    // 2. Boundary page lookups
    // Lowest numeric page in story: 000002
    let p_min_padded = store
        .get_story_page("000002")
        .await
        .expect("000002 must exist");
    assert_eq!(p_min_padded.page_id, "000002");
    assert_eq!(p_min_padded.title, "Start Jailbreak Adventure");

    let p_min_unpadded = store
        .get_story_page("2")
        .await
        .expect("Unpadded '2' must resolve to 000002");
    assert_eq!(p_min_unpadded.page_id, "000002");

    // Highest numeric page in story: 010030
    let p_max_padded = store
        .get_story_page("010030")
        .await
        .expect("010030 must exist");
    assert_eq!(p_max_padded.page_id, "010030");
    assert_eq!(p_max_padded.title, "[S] ==>");

    let p_max_unpadded = store
        .get_story_page("10030")
        .await
        .expect("Unpadded '10030' must resolve to 010030");
    assert_eq!(p_max_unpadded.page_id, "010030");

    // Non-existent boundary pages
    assert!(store.get_story_page("000000").await.is_none());
    assert!(store.get_story_page("000001").await.is_none()); // ryanquest, not story
    assert!(store.get_story_page("010031").await.is_none());
    assert!(store.get_story_page("10031").await.is_none());
    assert!(store.get_story_page("999999").await.is_none());
    assert!(store.get_story_page("-1").await.is_none());
    assert!(store.get_story_page("").await.is_none());
    assert!(store.get_story_page("   ").await.is_none());
    assert!(store.get_story_page("undefined").await.is_none());
    assert!(store.get_story_page("null").await.is_none());
    assert!(store.get_story_page("../etc/passwd").await.is_none());

    // 3. Boundary range lookups
    // Range at start (0)
    let range_zero = store.get_story_range(0, 5).await;
    assert_eq!(range_zero.len(), 5);
    assert_eq!(range_zero[0].page_id, "000002");
    assert_eq!(range_zero[1].page_id, "000003");

    // Range starting at page 2
    let range_page2 = store.get_story_range(2, 3).await;
    assert_eq!(range_page2.len(), 3);
    assert_eq!(range_page2[0].page_id, "000002");

    // Range reaching the end of numeric pages and crossing into non-numeric keys
    let range_end = store.get_story_range(10030, 10).await;
    assert_eq!(range_end.len(), 7); // 010030 + 6 non-numeric keys
    assert_eq!(range_end[0].page_id, "010030");
    let non_numeric_ids: Vec<String> = range_end[1..].iter().map(|s| s.page_id.clone()).collect();
    assert_eq!(
        non_numeric_ids,
        vec![
            "darkcage".to_string(),
            "darkcage2".to_string(),
            "jb2_000000".to_string(),
            "mc0001".to_string(),
            "pony".to_string(),
            "pony2".to_string(),
        ]
    );

    // 009991 is a valid page number in story (title "====>")
    let range_page_9991 = store.get_story_range(9991, 5).await;
    assert_eq!(range_page_9991.len(), 5);
    assert_eq!(range_page_9991[0].page_id, "009991");

    // Range at exact upper boundary: 10031 is the first non-existent page number,
    // and 10031 >= total (9991), so it returns empty.
    let range_past_total = store.get_story_range(10031, 5).await;
    assert!(range_past_total.is_empty());

    let range_far_beyond = store.get_story_range(999999, 10).await;
    assert!(range_far_beyond.is_empty());

    // Zero count range
    let range_zero_count = store.get_story_range(1901, 0).await;
    assert!(range_zero_count.is_empty());
}
