use std::{env, ffi::OsStr, ffi::OsString, os::windows::ffi::OsStrExt};

fn ascii_lower_wide(s: &OsStr) -> impl Iterator<Item=u16> + '_ {
    s.encode_wide().map(|c| {
        if (b'A' as u16..=b'Z' as u16).contains(&c) {
            c + 32
        } else {
            c
        }
    })
}

fn eq_ignore_ascii_case(a: &OsStr, b: &OsStr) -> bool {
    ascii_lower_wide(a).eq(ascii_lower_wide(b))
}

pub(crate) fn build_env_block(
    env_clear: bool,
    env_vars: Vec<(OsString, Option<OsString>)>,
) -> Option<Vec<u16>> {
    if !env_clear && env_vars.is_empty() {
        return None;
    }

    let mut map: Vec<(OsString, OsString)> = if env_clear {
        Vec::new()
    } else {
        env::vars_os().collect()
    };

    let mut seen: Vec<OsString> = Vec::new();
    for (key, val) in env_vars.into_iter().rev() {
        if seen.iter().any(|k| eq_ignore_ascii_case(k, &key)) {
            continue;
        }
        seen.push(key.clone());
        map.retain(|(k, _)| !eq_ignore_ascii_case(k, &key));
        if let Some(val) = val {
            map.push((key, val));
        }
    }

    let mut pairs: Vec<_> = map
        .drain(..)
        .map(|(key, val)| {
            let lowered: Vec<u16> = ascii_lower_wide(&key).collect();
            (lowered, key, val)
        })
        .collect();

    pairs.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
    map = pairs.into_iter().map(|(_, key, val)| (key, val)).collect();

    let mut block: Vec<u16> = Vec::new();
    for (key, val) in &map {
        block.extend(key.encode_wide());
        block.push(b'=' as u16);
        block.extend(val.encode_wide());
        block.push(0);
    }

    if map.is_empty() {
        block.extend(&[0, 0]);
    } else {
        block.push(0);
    }

    Some(block)
}
