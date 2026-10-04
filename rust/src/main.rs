use std::{
    env, fs,
    path::Path,
    process::{exit, Command},
};

type Cmd = &'static [&'static str];

struct Lang {
    id: &'static str,
    aliases: &'static [&'static str],
    /// Запускается в текущей папке и сам создаёт проект (cargo new, dotnet new).
    /// Если None — просто создаём папку и пишем `files`.
    scaffold: Option<Cmd>,
    files: &'static [(&'static str, &'static str)],
    /// Запускается внутри проекта после записи файлов (go mod init).
    setup: Option<Cmd>,
    build: Option<Cmd>,
    run: Cmd,
}

static LANGS: &[Lang] = &[
    Lang {
        id: "rust",
        aliases: &["rs"],
        scaffold: Some(&["cargo", "new", "{name}"]),
        files: &[],
        setup: None,
        build: Some(&["cargo", "build"]),
        run: &["cargo", "run"],
    },
    Lang {
        id: "c",
        aliases: &[],
        scaffold: None,
        files: &[(
            "main.c",
            "#include <stdio.h>\n\nint main(void) {\n    printf(\"Hello, world!\\n\");\n    return 0;\n}\n",
        )],
        setup: None,
        build: Some(&["gcc", "main.c", "-o", "{name}"]),
        run: &["./{name}"],
    },
    Lang {
        id: "cpp",
        aliases: &["c++", "cc"],
        scaffold: None,
        files: &[(
            "main.cpp",
            "#include <iostream>\n\nint main() {\n    std::cout << \"Hello, world!\\n\";\n    return 0;\n}\n",
        )],
        setup: None,
        build: Some(&["g++", "main.cpp", "-o", "{name}"]),
        run: &["./{name}"],
    },
    Lang {
        id: "zig",
        aliases: &[],
        scaffold: None,
        files: &[(
            "main.zig",
            "const std = @import(\"std\");\n\npub fn main() void {\n    std.debug.print(\"Hello, world!\\n\", .{});\n}\n",
        )],
        setup: None,
        build: Some(&["zig", "build-exe", "main.zig", "--name", "{name}"]),
        run: &["./{name}"],
    },
    Lang {
        id: "go",
        aliases: &["golang"],
        scaffold: None,
        files: &[(
            "main.go",
            "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello, world!\")\n}\n",
        )],
        setup: Some(&["go", "mod", "init", "{name}"]),
        build: Some(&["go", "build", "-o", "{name}", "."]),
        run: &["./{name}"],
    },
    Lang {
        id: "kotlin",
        aliases: &["kt"],
        scaffold: None,
        files: &[("main.kt", "fun main() {\n    println(\"Hello, world!\")\n}\n")],
        setup: None,
        build: Some(&["kotlinc", "main.kt", "-include-runtime", "-d", "{name}.jar"]),
        run: &["java", "-jar", "{name}.jar"],
    },
    Lang {
        id: "java",
        aliases: &[],
        scaffold: None,
        files: &[(
            "Main.java",
            "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, world!\");\n    }\n}\n",
        )],
        setup: None,
        build: Some(&["javac", "Main.java"]),
        run: &["java", "Main"],
    },
    Lang {
        id: "csharp",
        aliases: &["cs", "c#", "dotnet"],
        scaffold: Some(&["dotnet", "new", "console", "-o", "{name}"]),
        files: &[],
        setup: None,
        build: Some(&["dotnet", "build"]),
        run: &["dotnet", "run"],
    },
    Lang {
        id: "swift",
        aliases: &[],
        scaffold: None,
        files: &[("main.swift", "print(\"Hello, world!\")\n")],
        setup: None,
        build: Some(&["swiftc", "main.swift", "-o", "{name}"]),
        run: &["./{name}"],
    },
    Lang {
        id: "nim",
        aliases: &[],
        scaffold: None,
        files: &[("main.nim", "echo \"Hello, world!\"\n")],
        setup: None,
        build: Some(&["nim", "c", "-o:{name}", "main.nim"]),
        run: &["./{name}"],
    },
    Lang {
        id: "haskell",
        aliases: &["hs"],
        scaffold: None,
        files: &[("main.hs", "main :: IO ()\nmain = putStrLn \"Hello, world!\"\n")],
        setup: None,
        build: Some(&["ghc", "main.hs", "-o", "{name}"]),
        run: &["./{name}"],
    },
    Lang {
        id: "python",
        aliases: &["py"],
        scaffold: None,
        files: &[("main.py", "print(\"Hello, world!\")\n")],
        setup: None,
        build: None,
        run: &["python3", "main.py"],
    },
    Lang {
        id: "js",
        aliases: &["javascript", "node"],
        scaffold: None,
        files: &[("main.js", "console.log(\"Hello, world!\");\n")],
        setup: None,
        build: None,
        run: &["node", "main.js"],
    },
    Lang {
        id: "lua",
        aliases: &[],
        scaffold: None,
        files: &[("main.lua", "print(\"Hello, world!\")\n")],
        setup: None,
        build: None,
        run: &["lua", "main.lua"],
    },
    Lang {
        id: "dart",
        aliases: &[],
        scaffold: None,
        files: &[("main.dart", "void main() {\n  print('Hello, world!');\n}\n")],
        setup: None,
        build: None,
        run: &["dart", "run", "main.dart"],
    },
];

const USAGE: &str = "\
hw — генератор hello world проектов

Использование:
    hw <язык> [имя] [флаги]

Флаги:
    -b, --build    создать и собрать
    -r, --run      создать, собрать и запустить
    -l, --list     список языков
    -h, --help     эта справка

Примеры:
    hw rust
    hw cpp test -r
    hw zig --build";

fn find(query: &str) -> Option<&'static Lang> {
    let q = query.to_lowercase();
    LANGS.iter().find(|l| l.id == q || l.aliases.contains(&q.as_str()))
}

fn list() {
    for l in LANGS {
        if l.aliases.is_empty() {
            println!("{}", l.id);
        } else {
            println!("{} ({})", l.id, l.aliases.join(", "));
        }
    }
}

fn sh(cmd: Cmd, name: &str, dir: &Path) -> bool {
    let parts: Vec<String> = cmd.iter().map(|s| s.replace("{name}", name)).collect();
    println!("$ {}", parts.join(" "));
    match Command::new(&parts[0]).args(&parts[1..]).current_dir(dir).status() {
        Ok(s) if s.success() => true,
        Ok(s) => {
            eprintln!("команда завершилась с кодом {}", s.code().unwrap_or(-1));
            false
        }
        Err(e) => {
            eprintln!("не удалось запустить `{}`: {e}", parts[0]);
            false
        }
    }
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    exit(1);
}

fn main() {
    let mut build = false;
    let mut run = false;
    let mut pos: Vec<String> = Vec::new();

    for a in env::args().skip(1) {
        match a.as_str() {
            "-b" | "--build" => build = true,
            "-r" | "--run" => run = true,
            "-l" | "--list" => return list(),
            "-h" | "--help" => return println!("{USAGE}"),
            s if s.starts_with('-') => die(&format!("неизвестный флаг: {s}")),
            _ => pos.push(a),
        }
    }

    let Some(query) = pos.first() else {
        println!("{USAGE}");
        return;
    };
    let Some(lang) = find(query) else {
        eprintln!("не знаю язык `{query}`. Доступные:");
        list();
        exit(1);
    };

    let name = pos.get(1).cloned().unwrap_or_else(|| format!("hello-{}", lang.id));
    let cwd = env::current_dir().unwrap_or_else(|e| die(&format!("cwd: {e}")));
    let dir = cwd.join(&name);

    if dir.exists() {
        die(&format!("`{name}` уже существует"));
    }

    // создаём проект
    match lang.scaffold {
        Some(cmd) => {
            if !sh(cmd, &name, &cwd) {
                exit(1);
            }
        }
        None => fs::create_dir_all(&dir).unwrap_or_else(|e| die(&format!("mkdir: {e}"))),
    }
    for (file, content) in lang.files {
        fs::write(dir.join(file), content).unwrap_or_else(|e| die(&format!("{file}: {e}")));
    }
    if let Some(cmd) = lang.setup {
        if !sh(cmd, &name, &dir) {
            exit(1);
        }
    }
    println!("создан ./{name} [{}]", lang.id);

    // сборка / запуск
    if build || run {
        if let Some(cmd) = lang.build {
            if !sh(cmd, &name, &dir) {
                exit(1);
            }
        }
    }
    if run && !sh(lang.run, &name, &dir) {
        exit(1);
    }
}
