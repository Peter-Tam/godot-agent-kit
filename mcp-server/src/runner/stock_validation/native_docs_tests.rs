use super::*;

fn fixture(bytes: &[u8]) -> Result<(PrivateClone, Descriptor), &'static str> {
    let name = private_name()?;
    let private = private_clone_named(&name)?;
    let parent = artifact_parent(&private, true)?;
    let mut options = cap_std::fs::OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    parent
        .open_with(ARTIFACT, &options)
        .map_err(|_| "fixture")?
        .write_all(bytes)
        .map_err(|_| "fixture")?;
    let (size, sha256) = copy_artifact(
        &private,
        &mut io::sink(),
        None,
        Instant::now() + Duration::from_secs(2),
    )?;
    Ok((
        private,
        Descriptor {
            directory: name,
            size,
            sha256,
        },
    ))
}
fn destination() -> Result<PrivateClone, &'static str> {
    private_clone_named(&private_name()?)
}
fn at() -> Instant {
    Instant::now() + Duration::from_secs(2)
}

#[test]
fn missing_changed_truncated_and_oversized_artifacts_refuse() -> Result<(), &'static str> {
    for case in 0..4 {
        let (source, descriptor) = fixture(b"native-documents")?;
        let parent = artifact_parent(&source, false)?;
        match case {
            0 => parent.remove_file(ARTIFACT).map_err(|_| "fixture")?,
            1 => {
                parent
                    .write(ARTIFACT, b"Native-documents")
                    .map_err(|_| "fixture")?;
            }
            2 => {
                parent.write(ARTIFACT, b"native").map_err(|_| "fixture")?;
            }
            _ => {
                let mut options = cap_std::fs::OpenOptions::new();
                options.write(true);
                parent
                    .open_with(ARTIFACT, &options)
                    .map_err(|_| "fixture")?
                    .set_len(MAX_ARTIFACT + 1)
                    .map_err(|_| "fixture")?;
            }
        }
        let target = destination()?;
        assert!(seed(&target, &descriptor, at()).is_err());
        assert!(!artifact_parent(&target, false)?.exists(ARTIFACT));
        assert!(
            source.0.exists(),
            "borrowed source must not be removed on refusal"
        );
    }
    Ok(())
}

#[test]
fn descriptor_bounds_namespace_and_deadline_refuse() -> Result<(), &'static str> {
    let (_source, mut descriptor) = fixture(b"native-documents")?;
    descriptor.size = MAX_ARTIFACT + 1;
    assert!(seed(&destination()?, &descriptor, at()).is_err());
    descriptor.size = 16;
    descriptor.directory = "../other-home".into();
    assert!(seed(&destination()?, &descriptor, at()).is_err());
    let (_valid_source, descriptor) = fixture(b"native-documents")?;
    assert!(seed(&destination()?, &descriptor, Instant::now()).is_err());
    Ok(())
}

#[test]
fn seed_creates_independent_private_bytes_and_refuses_existing_file() -> Result<(), &'static str> {
    let (source, descriptor) = fixture(b"native-documents")?;
    let target = destination()?;
    seed(&target, &descriptor, at())?;
    let source_parent = artifact_parent(&source, false)?;
    let target_parent = artifact_parent(&target, false)?;
    let source_meta = source_parent.metadata(ARTIFACT).map_err(|_| "fixture")?;
    let target_meta = target_parent.metadata(ARTIFACT).map_err(|_| "fixture")?;
    assert_ne!(
        (source_meta.dev(), source_meta.ino()),
        (target_meta.dev(), target_meta.ino())
    );
    assert_eq!(target_meta.nlink(), 1);
    assert_eq!(target_meta.permissions().mode() & 0o777, 0o600);
    assert!(seed(&target, &descriptor, at()).is_err());
    target_parent
        .write(ARTIFACT, b"modified-target!")
        .map_err(|_| "fixture")?;
    assert_eq!(
        source_parent.read(ARTIFACT).map_err(|_| "fixture")?,
        b"native-documents"
    );
    assert!(seed(&source, &descriptor, at()).is_err());
    Ok(())
}

#[test]
fn symlink_and_hardlink_artifacts_refuse() -> Result<(), &'static str> {
    let (source, descriptor) = fixture(b"native-documents")?;
    let parent = artifact_parent(&source, false)?;
    parent
        .hard_link(ARTIFACT, &parent, "alias")
        .map_err(|_| "fixture")?;
    assert!(seed(&destination()?, &descriptor, at()).is_err());
    parent.remove_file("alias").map_err(|_| "fixture")?;
    parent
        .rename(ARTIFACT, &parent, "real")
        .map_err(|_| "fixture")?;
    parent.symlink("real", ARTIFACT).map_err(|_| "fixture")?;
    assert!(seed(&destination()?, &descriptor, at()).is_err());
    Ok(())
}

#[test]
fn weak_registration_concurrent_borrowers_keep_snapshot_until_last_drop() -> Result<(), &'static str>
{
    let (private, descriptor) = fixture(b"native-documents")?;
    let path = private.0.clone();
    let owner = Arc::new(PreparedDocuments {
        _private: private,
        descriptor,
        elapsed_us: 1,
    });
    let registration = Registration::Ready(Arc::downgrade(&owner));
    let first = borrow_registered(&registration)?.ok_or("fixture")?;
    let second = borrow_registered(&registration)?.ok_or("fixture")?;
    drop(owner);
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let mut threads = Vec::new();
    for borrower in [first, second] {
        let barrier = barrier.clone();
        threads.push(thread::spawn(move || -> Result<(), &'static str> {
            let result =
                destination().and_then(|target| seed(&target, &borrower.descriptor(), at()));
            barrier.wait();
            barrier.wait();
            drop(borrower);
            result
        }));
    }
    barrier.wait();
    assert!(path.exists());
    assert!(borrow_registered(&registration)?.is_some());
    barrier.wait();
    for worker in threads {
        worker.join().map_err(|_| "thread_failure")??;
    }
    assert!(!path.exists());
    assert!(borrow_registered(&registration).is_err());
    assert!(borrow_registered(&Registration::Never)?.is_none());
    assert!(borrow_registered(&Registration::Preparing).is_err());
    assert!(borrow_registered(&Registration::Failed).is_err());
    Ok(())
}
