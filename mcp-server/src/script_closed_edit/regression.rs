use super::*;
fn basis(present: bool) -> ClosedExpectedBasis {
    let digest = "a".repeat(64);
    ClosedExpectedBasis {
        prior_request: "read".into(),
        project: "/project".into(),
        session: "a".repeat(32),
        path: "res://a.gd".into(),
        state: ExpectedState {
            project_device: "1".into(),
            project_inode: "2".into(),
            close_epoch: "3".into(),
            file_revision: FileRevision {
                device: "1".into(),
                inode: "4".into(),
                utf8_bytes: "0".into(),
                sha256: digest.clone(),
                mtime: Timestamp {
                    seconds: "1".into(),
                    nanoseconds: 2,
                },
                ctime: Timestamp {
                    seconds: "3".into(),
                    nanoseconds: 4,
                },
            },
            resource: ExpectedResource {
                state: if present { "present" } else { "absent" }.into(),
                instance_id: present.then(|| "5".into()),
                path: present.then(|| "res://a.gd".into()),
                source_sha256: present.then(|| digest.clone()),
                utf8_bytes: present.then(|| "0".into()),
                edited: present.then_some(false),
                profile_sha256: present.then(|| "b".repeat(64)),
            },
        },
    }
}
fn commitment(b: &ClosedExpectedBasis) -> Vec<u8> {
    let mut c = ring::digest::Context::new(&ring::digest::SHA256);
    b.stable_commitment(&mut c);
    c.finish().as_ref().to_vec()
}
#[test]
fn commitment_binds_every_closed_scalar_and_excludes_correlation() {
    let original = basis(true);
    let before = commitment(&original);
    let mut incidental = original.clone();
    incidental.prior_request = "different-read".into();
    assert_eq!(before, commitment(&incidental));
    let mutations: [fn(&mut ClosedExpectedBasis); 21] = [
        |b| b.project.push('x'),
        |b| b.session.push('x'),
        |b| b.path.push('x'),
        |b| b.state.project_device.push('1'),
        |b| b.state.project_inode.push('1'),
        |b| b.state.close_epoch.push('1'),
        |b| b.state.file_revision.device.push('1'),
        |b| b.state.file_revision.inode.push('1'),
        |b| b.state.file_revision.utf8_bytes.push('1'),
        |b| b.state.file_revision.sha256.replace_range(0..1, "b"),
        |b| b.state.file_revision.mtime.seconds.push('1'),
        |b| b.state.file_revision.ctime.seconds.push('1'),
        |b| b.state.file_revision.mtime.nanoseconds += 1,
        |b| b.state.file_revision.ctime.nanoseconds += 1,
        |b| b.state.resource.state = "absent".into(),
        |b| b.state.resource.instance_id = Some("6".into()),
        |b| b.state.resource.path = Some("res://b.gd".into()),
        |b| b.state.resource.source_sha256 = Some("c".repeat(64)),
        |b| b.state.resource.utf8_bytes = Some("1".into()),
        |b| b.state.resource.edited = Some(true),
        |b| b.state.resource.profile_sha256 = Some("c".repeat(64)),
    ];
    for mutate in mutations {
        let mut b = original.clone();
        mutate(&mut b);
        assert_ne!(before, commitment(&b));
    }
    assert_ne!(commitment(&basis(false)), before);
}
#[test]
fn absent_metadata_requires_explicit_null_facts() {
    assert!(serde_json::from_str::<ExpectedResource>(r#"{"state":"absent"}"#).is_err());
    let absent = basis(false);
    assert!(absent.state.valid(&absent.path));
    let mut unsupported = absent.clone();
    unsupported.state.file_revision.utf8_bytes = (SOURCE_LIMIT_BYTES + 1).to_string();
    assert!(!unsupported.state.valid(&unsupported.path));
}
#[test]
fn expected_basis_never_serializes_source_text() {
    let s = State {
        project_device: "1".into(),
        project_inode: "2".into(),
        close_epoch: "3".into(),
        lifecycle: "closed".into(),
        file_revision: basis(true).state.file_revision,
        resource: ResourceState {
            state: "present".into(),
            instance_id: Some("4".into()),
            path: Some("res://a.gd".into()),
            source: Some("private-source".into()),
            edited: Some(false),
            profile_sha256: Some("b".repeat(64)),
        },
    };
    assert!(!serde_json::to_string(&s.expected())
        .unwrap()
        .contains("private-source"));
}
#[test]
fn matching_read_correlation_cannot_become_mutation_id() {
    let b = basis(false);
    assert!(matches!(
        CheckedClosedRequest::new(
            RequestId::new("read").unwrap(),
            b,
            ReplacementSource::new(String::new()).unwrap()
        ),
        Err(BasisError::Correlation)
    ));
}

#[test]
fn incomplete_or_unsafe_resource_evidence_never_authorizes_a_closed_edit() {
    let source = "extends Node\n";
    let mut state = State {
        project_device: "1".into(),
        project_inode: "2".into(),
        close_epoch: "3".into(),
        lifecycle: "closed".into(),
        file_revision: basis(true).state.file_revision,
        resource: ResourceState {
            state: "present".into(),
            instance_id: Some("5".into()),
            path: Some("res://a.gd".into()),
            source: Some(source.into()),
            edited: Some(false),
            profile_sha256: Some("b".repeat(64)),
        },
    };
    state.file_revision.sha256 = confined::hex_sha256(source.as_bytes());
    state.file_revision.utf8_bytes = source.len().to_string();
    assert!(state.valid("res://a.gd"));
    assert!(state.expected().valid("res://a.gd"));
    state.resource.edited = Some(true);
    assert!(state.valid("res://a.gd"));
    assert!(!state.expected().valid("res://a.gd"));
    state.resource.edited = Some(false);
    state.resource.source = Some("extends RefCounted\n".into());
    assert!(state.valid("res://a.gd"));
    assert!(!state.expected().valid("res://a.gd"));
    state.resource.source = Some(source.into());
    state.resource.profile_sha256 = None;
    assert!(state.valid("res://a.gd"));
    assert!(!state.expected().valid("res://a.gd"));
}
