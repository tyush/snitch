use regex::{Regex, RegexBuilder};
use std::{
    error::Error,
    fmt::Display,
    fs::read_to_string,
    path::{Path, PathBuf},
    usize::MIN,
};
use walkdir::{DirEntry, WalkDir};

lazy_static::lazy_static! {
    static ref TODO_EXPR: Regex = RegexBuilder::new(r"^\s*(\S+\s)todo*: (.*)$").case_insensitive(true).build().unwrap();
    static ref TODO_PRI_EXPR: Regex = RegexBuilder::new(r"todo+").case_insensitive(true).build().unwrap();
}

pub fn find_all_todo_lines(source: &Vec<String>) -> Vec<usize> {
    source
        .iter()
        .enumerate()
        .filter_map(|(row, line)| TODO_EXPR.is_match(line).then(|| row))
        .collect()
}

pub fn sort_todos_by_priority(todos: &mut Vec<Todo>) {
    todos.sort_by_cached_key(|a| measure_priority(&a.message));
    // swap b/a to sort descending
    // actually, don't reverse. this way, highest priority is closest to bottom of screen
}

pub fn measure_priority(s: &str) -> usize {
    if let Some(m) = TODO_PRI_EXPR.find(s) {
        m.end() - m.start()
    } else {
        MIN
    }
}

pub fn find_files(path: &Path) -> Vec<DirEntry> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|f| {
            let entry = f.ok()?;
            entry.metadata().ok()?.is_file().then(|| entry)
        })
        .collect()
}

pub struct Todo {
    message: String,
    file: String,
    line: usize,
    col: usize,
}

// impl Todo {
//     pub fn get_priority(&self) -> usize {
//         measure_priority(&self.message)
//     }
// }

impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            // print file name, line#, col#. go back to left side. tab x4. print todo
            "{}:{}:{}\r\t\t\t\t{}",
            self.file, self.line, self.col, self.message.split_at(TODO_PRI_EXPR.find(&self.message).unwrap().start()).1
        ))
    }
}

pub fn scan(path: String) -> Result<(), Box<dyn Error>> {
    let files = find_files(Path::new(&path));

    let files: Vec<Box<PathBuf>> = files
        .iter()
        .map(|f| Box::new(f.path().to_owned()))
        .collect();

    let mut cant_read: Vec<String> = vec![];

    let mut todos_per_file = vec![];

    for file in files {
        let file = file.as_path();
        let lines = read_to_string(file);

        if let Ok(glob) = lines {
            let lines: Vec<String> = glob.split('\n').map(|s| s.to_owned()).collect();

            let mut zipped = vec![];

            for line_number in find_all_todo_lines(&lines) {
                zipped.push(Todo {
                    file: file
                        .to_str()
                        .expect("Found file, but can't print name due to invalid UTF-8!")
                        .to_owned(),
                    message: lines.get(line_number).unwrap().to_owned(),
                    line: line_number,
                    col: TODO_PRI_EXPR
                        .find(lines.get(line_number).unwrap())
                        .unwrap()
                        .start(),
                })
            }

            todos_per_file.push(zipped);
        } else {
            cant_read.push(
                file.to_str()
                    .expect("Found file, but can't print name due to invalid UTF-8!")
                    .to_owned(),
            );
        }
    }

    let mut todos_per_file = todos_per_file.into_iter().flatten().collect();
    sort_todos_by_priority(&mut todos_per_file);

    println!(
        "{}",
        todos_per_file
            .iter()
            .map(|t| format!("{}", t) + "\n")
            .collect::<String>()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{find_all_todo_lines, sort_todos_by_priority, Todo, measure_priority};

    #[test]
    fn finds_todos_in_text() {
        let source = vec![
            "// TODO: Rewrite this in haskell",                 // line 0
            "// todo: fire the guy who wrote previous todo",    // line 1
            "// todoooooooo: reduce hostility of workspace",    // line 2
            "todo: Hey guys can we not argue in our commits?",  // line 3
            "       // Todo: Teach management to use comments", // line 4
        ]
        .iter()
        .map(|&s| s.to_owned())
        .collect();

        assert_eq!(find_all_todo_lines(&source), vec![0, 1, 2, 4])
    }

    #[test]
    fn measure_priority_test() {
        assert_eq!(measure_priority("// TODO: fdjsafdasfda"), 4);
        assert_eq!(
            measure_priority("// ToDooOOo: Find A BetTer CaPiTalIZaTion SysTem"),
            8
        );
        assert_eq!(
            measure_priority("haha fool this ain't got a to do!!!!!"),
            std::usize::MIN
        );
    }

    #[test]
    fn does_sort_todos_by_priority() {
        let mut todos: Vec<Todo> = vec![
            Todo {
                file: "test_file.rs".to_owned(),
                col: 1,
                line: 12,
                message: "// todooo: fjdklas".to_owned()
            },
            Todo {
                file: "test_file.rs".to_owned(),
                col: 1,
                line: 63,
                message: "// todo: fdjskalfhdjasklfhjdasklf".to_owned()
            },
            Todo {
                file: "test_file.rs".to_owned(),
                col: 1,
                line: 42,
                message: "// todooooooo: fdjsal;fdddd".to_owned()
            },
        ];
        sort_todos_by_priority(&mut todos);

        assert_eq!(todos[0].line, 63);
        assert_eq!(todos[1].line, 12);
        assert_eq!(todos[2].line, 42);
    }

    // todo: find a way to test find_files reliably (maybe /test/ dir?)
}
