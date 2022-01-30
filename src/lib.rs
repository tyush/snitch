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
    static ref TODO_EXPR: Regex = RegexBuilder::new(r"^\s*(\S+\s)todo*:? (.*)$").case_insensitive(true).build().unwrap();
    static ref TODO_PRI_EXPR: Regex = RegexBuilder::new(r"todo+").case_insensitive(true).build().unwrap();
}

pub fn find_all_todo_lines(source: &Vec<String>) -> Vec<usize> {
    source
        .iter()
        .enumerate()
        .filter(|(_, line)| TODO_EXPR.is_match(line))
        .unzip::<usize, &String, Vec<usize>, Vec<_>>()
        .0
}

pub fn sort_todos_by_priority(todos: &mut Vec<(usize, String)>) {
    todos.sort_by(|(_, a), (_, b)| measure_priority(b).cmp(&measure_priority(a)));
    // swap b/a to sort descending
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
            if let Ok(entry) = f {
                if let Ok(m) = entry.metadata() {
                    if m.is_file() {
                        return Some(entry)
                    }
                }
            };
            None
        })
        .collect()
}

struct Todo {
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

    println!(
        "{}",
        todos_per_file
            .iter()
            .flatten()
            .map(|t| format!("{}", t) + "\n")
            .collect::<String>()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{find_all_todo_lines, sort_todos_by_priority};

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
        // todo: *haha irony* find out why this function requires an import but others don't
        use crate::measure_priority;
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
        let mut todos: Vec<(usize, String)> = vec![
            (42, "// TODO: move this to line 43".to_owned()),
            (63, "// TODOOOOOOOOOOOO: move this to line 42".to_owned()),
            (12, "// TODOOO: fire whoever made our program file size affect runtime".to_owned())
        ];
        sort_todos_by_priority(&mut todos);

        assert_eq!(todos[0].0, 63);
        assert_eq!(todos[1].0, 12);
        assert_eq!(todos[2].0, 42);
    }

    // todo: find a way to test find_files reliably (maybe /test/ dir?)
}
