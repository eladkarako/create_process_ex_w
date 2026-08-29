use std::{env, ffi::OsStr, ffi::OsString, os::windows::ffi::OsStrExt};

/// ASCII-lowercase a wide representation of an `OsStr`.
///
/// Windows environment variable handling is case-insensitive for ASCII letters in
/// practice, and the rest of this module uses a case-insensitive comparison strategy.
///
/// This function:
/// - encodes the `OsStr` as UTF-16 code units
/// - converts ASCII A-Z to a-z by adding 32
///
/// It does not attempt to apply full Unicode case folding—only ASCII is normalized.
fn ascii_lower_wide(s: &OsStr) -> impl Iterator<Item=u16> + '_ {
    s.encode_wide().map(|c| {
        if (b'A' as u16..=b'Z' as u16).contains(&c) {
            c + 32
        } else {
            c
        }
    })
}

/// Compares two environment variable keys ignoring ASCII-case.
///
/// This is used to detect whether the user is overriding/removing a variable that
/// already exists in the current environment.
///
/// # Returns
/// - `true` if `a` and `b` match after ASCII-lowercasing their UTF-16 encodings.
/// - `false` otherwise.
fn eq_ignore_ascii_case(a: &OsStr, b: &OsStr) -> bool {
    ascii_lower_wide(a).eq(ascii_lower_wide(b))
}

/// Builds a Windows environment block suitable for `CreateProcessW`.
///
/// Windows expects an environment block as a sequence of null-terminated strings
/// in the form `KEY=VALUE`, terminated by an extra null character (i.e., two
/// consecutive `\0` UTF-16 code units) to mark the end of the block.
///
/// This function returns:
/// - `None` when no custom environment is needed (meaning the caller should pass a
///   `NULL` environment pointer to Windows).
/// - `Some(Vec<u16>)` containing the properly formatted environment block (UTF-16)
///   when either:
///   - `env_clear` is `true`, or
///   - `env_vars` is non-empty.
///
/// # Parameters
/// - `env_clear`: If `true`, start from an empty environment map and only apply
///   `env_vars`.
/// - `env_vars`: A list of environment modifications:
///   - `(key, Some(val))` sets/overrides `key` with `val`.
///   - `(key, None)` removes/unsets `key`.
///
/// # Behavior details
/// - Keys are treated as case-insensitive using ASCII-case-insensitive matching.
/// - When multiple entries for the same key exist in `env_vars`, the *last* one
///   provided by the caller wins.
/// - The function normalizes keys (lowercases them for sorting and stability) when
///   ordering the final output.
///
/// # Returns
/// - `None` if `env_clear` is `false` and `env_vars` is empty.
/// - `Some(block)` otherwise, where `block` is UTF-16 code units containing
///   `KEY=VALUE\0 ... \0\0`.
pub(crate) fn build_env_block(
    env_clear: bool,
    env_vars: Vec<(OsString, Option<OsString>)>,
) -> Option<Vec<u16>> {
    // If we didn't ask to clear the environment and the user didn't specify any changes,
    // there is nothing to build. The caller should pass a NULL env pointer.
    if !env_clear && env_vars.is_empty() {
        return None;
    }

    // Initialize the base environment map:
    // - empty when env_clear is requested
    // - otherwise, current process environment.
    let mut map: Vec<(OsString, OsString)> = if env_clear {
        Vec::new()
    } else {
        env::vars_os().collect()
    };

    // Track which keys we have already handled from `env_vars`, to ensure "last wins".
    // We iterate `env_vars` in reverse and skip duplicates based on ASCII-case-insensitive
    // key comparison.
    let mut seen: Vec<OsString> = Vec::new();

    for (key, val) in env_vars.into_iter().rev() {
        // If we've already applied a modification for this key (in a later position),
        // ignore earlier duplicates.
        if seen.iter().any(|k| eq_ignore_ascii_case(k, &key)) {
            continue;
        }

        // Mark key as handled.
        seen.push(key.clone());

        // Remove any existing entry matching this key (case-insensitive).
        map.retain(|(k, _)| !eq_ignore_ascii_case(k, &key));

        // If the modification is a set/override, add the new key/value pair.
        if let Some(val) = val {
            map.push((key, val));
        }
    }

    // Prepare pairs for deterministic ordering:
    // - compute a lowered (ASCII-normalized) UTF-16 key for sorting
    // - keep the original key/value for actual output
    let mut pairs: Vec<_> = map
        .drain(..)
        .map(|(key, val)| {
            let lowered: Vec<u16> = ascii_lower_wide(&key).collect();
            (lowered, key, val)
        })
        .collect();

    // Sort by lowered key so output is stable regardless of original key casing.
    pairs.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));

    // Restore (key, val) pairs in sorted order.
    map = pairs.into_iter().map(|(_, key, val)| (key, val)).collect();

    // Serialize the environment block:
    // For each KEY=VALUE pair, append:
    // - KEY UTF-16
    // - '=' separator (as UTF-16)
    // - VALUE UTF-16
    // - null terminator (0)
    //
    // After all pairs, append a final null character to mark the end of the block.
    let mut block: Vec<u16> = Vec::new();
    for (key, val) in &map {
        block.extend(key.encode_wide());
        block.push(b'=' as u16);
        block.extend(val.encode_wide());
        block.push(0);
    }

    // Windows requires an additional null terminator after the last KEY=VALUE\0.
    // This results in:
    // - empty map: [0, 0] (immediate double-null)
    // - non-empty map: one extra 0 after the last string terminator
    if map.is_empty() {
        block.extend(&[0, 0]);
    } else {
        block.push(0);
    }

    Some(block)
}
