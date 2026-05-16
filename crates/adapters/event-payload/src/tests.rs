use super::*;

#[test]
fn thought_created_roundtrip() {
    let p = EventPayload::ThoughtCreated {
        thought_id: "abc".into(),
        user_id: "def".into(),
        in_reply_to_id: None,
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: EventPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(back.subject(), "thoughts.created");
}

#[test]
fn all_subjects_are_unique() {
    let samples: &[EventPayload] = &[
        EventPayload::ThoughtCreated {
            thought_id: "a".into(),
            user_id: "b".into(),
            in_reply_to_id: None,
        },
        EventPayload::ThoughtDeleted {
            thought_id: "a".into(),
            user_id: "b".into(),
        },
        EventPayload::ThoughtUpdated {
            thought_id: "a".into(),
            user_id: "b".into(),
        },
        EventPayload::LikeAdded {
            like_id: "a".into(),
            user_id: "b".into(),
            thought_id: "c".into(),
        },
        EventPayload::LikeRemoved {
            user_id: "b".into(),
            thought_id: "c".into(),
        },
        EventPayload::BoostAdded {
            boost_id: "a".into(),
            user_id: "b".into(),
            thought_id: "c".into(),
        },
        EventPayload::BoostRemoved {
            user_id: "b".into(),
            thought_id: "c".into(),
        },
        EventPayload::FollowRequested {
            follower_id: "a".into(),
            following_id: "b".into(),
        },
        EventPayload::FollowAccepted {
            follower_id: "a".into(),
            following_id: "b".into(),
        },
        EventPayload::FollowRejected {
            follower_id: "a".into(),
            following_id: "b".into(),
        },
        EventPayload::Unfollowed {
            follower_id: "a".into(),
            following_id: "b".into(),
        },
        EventPayload::UserBlocked {
            blocker_id: "a".into(),
            blocked_id: "b".into(),
        },
        EventPayload::UserUnblocked {
            blocker_id: "a".into(),
            blocked_id: "b".into(),
        },
        EventPayload::UserRegistered {
            user_id: "a".into(),
        },
    ];
    let mut subjects: Vec<&str> = samples.iter().map(|p| p.subject()).collect();
    subjects.sort();
    subjects.dedup();
    assert_eq!(
        subjects.len(),
        samples.len(),
        "each event must have a unique subject"
    );
}
