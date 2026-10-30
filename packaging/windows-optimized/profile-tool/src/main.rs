#![cfg_attr(windows, windows_subsystem = "windows")]
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const FILES: [&str; 3] = [
    "game/options.txt",
    "game/horizon-graphics.txt",
    "launcher/settings.json",
];
const OPTIONS: &[(&str, &str)] = &[
    ("maxFps", "60"),
    ("enableVsync", "false"),
    ("renderDistance", "6"),
    ("simulationDistance", "5"),
    ("menuBackgroundBlurriness", "0"),
    ("panoramaScrollSpeed", "0"),
    ("overrideWidth", "960"),
    ("overrideHeight", "540"),
    ("entityShadows", "false"),
    ("entityDistanceScaling", "0.5"),
    ("biomeBlendRadius", "0"),
    ("mipmapLevels", "0"),
    ("particles", "2"),
    ("renderClouds", "false"),
    ("cutoutLeaves", "false"),
];
const GRAPHICS: &[(&str, &str)] = &[
    ("graphicsApi", "opengl"),
    ("framePacing", "efficient"),
    ("workers", "2"),
    ("chunkUpdates", "4"),
    ("uploadBudgetKiB", "512"),
    ("lodDistance", "0"),
    ("fxaa", "false"),
    ("taa", "false"),
    ("hdr", "false"),
    ("resolutionScale", "100"),
    ("upscaling", "native"),
    ("pickedShaderPack", ""),
];

fn merge(text: &str, changes: &[(&str, &str)]) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if !changes.iter().any(|(key, _)| {
            line.strip_prefix(key)
                .is_some_and(|tail| tail.starts_with(':'))
        }) {
            out.push_str(line);
            out.push('\n');
        }
    }
    for (key, value) in changes {
        out.push_str(&format!("{key}:{value}\n"));
    }
    out
}

fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("Missing parent")?).map_err(|e| e.to_string())?;
    let temp = path.with_extension("light-tmp");
    fs::write(&temp, bytes)
        .and_then(|_| fs::rename(&temp, path))
        .map_err(|e| e.to_string())
}

fn profile(root: &Path, restore: bool) -> Result<(), String> {
    let data = root.join("data");
    let backup = data.join("before-light-profile");
    let state_file = backup.join("state.json");
    if restore {
        let state: Value =
            serde_json::from_slice(&fs::read(&state_file).map_err(|_| "Não há backup anterior")?)
                .map_err(|e| e.to_string())?;
        for (i, name) in FILES.iter().enumerate() {
            let path = data.join(name);
            if state[i].as_bool() == Some(true) {
                atomic(
                    &path,
                    &fs::read(backup.join(i.to_string())).map_err(|e| e.to_string())?,
                )?;
            } else if path.exists() {
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
        }
        return Ok(());
    }
    let old: Vec<Option<Vec<u8>>> = FILES
        .iter()
        .map(|name| match fs::read(data.join(name)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.to_string()),
        })
        .collect::<Result<_, _>>()?;
    let text = |i: usize| -> Result<&str, String> {
        std::str::from_utf8(old[i].as_deref().unwrap_or_default()).map_err(|e| e.to_string())
    };
    let options = merge(text(0)?, OPTIONS);
    let graphics = merge(text(1)?, GRAPHICS);
    let mut settings: Value = if old[2].is_some() {
        serde_json::from_str(text(2)?).map_err(|e| e.to_string())?
    } else {
        json!({})
    };
    if !settings.is_object() {
        return Err("settings.json precisa ser um objeto".into());
    }
    settings["skip_opening"] = json!(true);
    settings["while_playing"] = json!("minimize");
    settings["install_new_versions"] = json!(false);
    if !state_file.exists() {
        fs::create_dir_all(&backup).map_err(|e| e.to_string())?;
        for (i, bytes) in old.iter().enumerate() {
            if let Some(bytes) = bytes {
                fs::write(backup.join(i.to_string()), bytes).map_err(|e| e.to_string())?;
            }
        }
        atomic(
            &state_file,
            &serde_json::to_vec(&old.iter().map(Option::is_some).collect::<Vec<_>>()).unwrap(),
        )?;
    }
    for (name, bytes) in FILES.iter().zip([
        options.into_bytes(),
        graphics.into_bytes(),
        serde_json::to_vec_pretty(&settings).unwrap(),
    ]) {
        atomic(&data.join(name), &bytes)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let result = if args.len() == 2 {
        profile(&PathBuf::from(&args[1]), args[0] == "restore")
    } else {
        Err("Uso: apply|restore PASTA".into())
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_unknown_keys_and_unicode() {
        let s = merge(
            "key.jump:key.keyboard.x\nresourcePacks:[\"árvores\"]\nmaxFps:260\nmaxFps:120\n",
            OPTIONS,
        );
        assert!(s.contains("key.jump:key.keyboard.x"));
        assert!(s.contains("árvores"));
        assert_eq!(s.matches("maxFps:").count(), 1);
        assert!(s.contains("maxFps:60"));
    }
    #[test]
    fn apply_restore_preserves_exact_originals_and_missing_files() {
        let root =
            std::env::temp_dir().join(format!("horizon-light-profile-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("data/game/saves/Meu Mundo")).unwrap();
        fs::write(
            root.join("data/game/saves/Meu Mundo/level.dat"),
            b"untouched",
        )
        .unwrap();
        let original = b"maxFps:120\r\nkey.jump:key.keyboard.x\r\n";
        fs::write(root.join("data/game/options.txt"), original).unwrap();
        profile(&root, false).unwrap();
        profile(&root, false).unwrap();
        profile(&root, true).unwrap();
        assert_eq!(
            fs::read(root.join("data/game/options.txt")).unwrap(),
            original
        );
        assert!(!root.join("data/game/horizon-graphics.txt").exists());
        assert!(!root.join("data/launcher/settings.json").exists());
        assert_eq!(
            fs::read(root.join("data/game/saves/Meu Mundo/level.dat")).unwrap(),
            b"untouched"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
