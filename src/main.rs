use std::env;
use std::fmt;
use std::fs::{read_to_string, File, OpenOptions};
use std::io::{self, Write};

use uuid::Uuid;

const TODO_PATH: &str = "./base.todo";

#[derive(Clone, Debug)]
struct TodoInput {
    name: String,
    description: String,
    status: TodoStatus,
}

#[derive(Clone, Debug)]
struct TodoUpdateInput {
    name: Option<String>,
    description: Option<String>,
    status: Option<TodoStatus>,
}

#[derive(Clone, Debug)]
struct Todo {
    id: String,
    todo: TodoInput,
}

// ファイルを、ヘッダを挿入した状態で作成する
fn create_file(path: &str) -> io::Result<File> {
    match File::create(path) {
	Ok(file) => {
	    let mut f = file;
	    let _ = f.write_all(b"id,name,status,description\n");
	    Ok(f)
	}
	Err(err) => Err(err),
    }
}

fn read_file_or_create(path: &str, is_append: bool) -> io::Result<File> {
    if is_append {
	match OpenOptions::new().write(true).truncate(true).open(path) {
	    Ok(file) => Ok(file),
	    Err(_) => {
		let file = File::create(path);
		match file {
		    Ok(f) => {
			let mut ff = f;
			let _ = ff.write_all(b"id,name,status,description\n");
			Ok(ff)
		    }
		    Err(err) => Err(err),
		}
	    }
	}
    } else {
	match File::open(path) {
	    Ok(file) => Ok(file),
	    Err(_) => match create_file(path) {
		Ok(f) => Ok(f),
		Err(err) => Err(err),
	    },
	}
    }
}

#[derive(Clone, Copy, Debug)]
enum TodoStatus {
    Todo = 1,
    Doing = 2,
    Doit = 3,
}

impl fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	match self {
	    TodoStatus::Todo => write!(f, "Todo"),
	    TodoStatus::Doing => write!(f, "Doing"),
	    TodoStatus::Doit => write!(f, "Doit"),
	}
    }
}

impl TodoStatus {
    fn from_usize(n: usize) -> Option<TodoStatus> {
	match n {
	    1 => Some(TodoStatus::Todo),
	    2 => Some(TodoStatus::Doing),
	    3 => Some(TodoStatus::Doit),
	    _ => None,
	}
    }
}

impl Todo {
    fn line(&self) -> String {
	self.todo.line(&self.id)
    }
}

impl TodoInput {
    fn line(&self, id: &str) -> String {
	format!(
	    "{},{},{},{}",
	    id, self.name, self.status as usize, self.description
	)
    }
}

fn create(input: TodoInput) -> Result<String, String> {
    match read_file_or_create(TODO_PATH, true).as_mut() {
	Ok(file) => {
	    let id = Uuid::new_v4();
	    let line = input.line(&id.to_string());
	    match file.write(line.as_bytes()) {
		Ok(_) => Ok(id.to_string()),
		Err(e) => Err(format!("Error: {}", e)),
	    }
	}
	Err(_) => Err(format!("create error.")),
    }
}
fn update(target_id: &str, input: TodoUpdateInput) -> Result<&str, String> {
    let mut result: String = format!("id,name,status,description\n");
    let status = input.status;

    for line in read_to_string(TODO_PATH).unwrap().lines() {
	let items: Vec<&str> = line.split(",").collect();
	if items.len() != 4 {
	    continue;
	}
	let id: String = items[0].to_string();
	let stt: usize = items[2].parse().unwrap();
	let st: TodoStatus = TodoStatus::from_usize(stt).unwrap();
	let line: String = match target_id == id {
	    true => {
		let input = Todo {
		    id,
		    todo: TodoInput {
			name: input.name.clone().unwrap_or(items[1].to_string()),
			status: status.unwrap_or(st),
			description: input.description.clone().unwrap_or(items[3].to_string()),
		    },
		};
		input.line()
	    }
	    _ => {
		let input = Todo {
		    id,
		    todo: TodoInput {
			name: items[1].to_string(),
			status: st,
			description: items[3].to_string(),
		    },
		};
		input.line()
	    }
	};
	result = format!("{}\n{}", result, line);
    }
    let rb = result.as_bytes();
    match File::open(TODO_PATH) {
	Ok(f) => {
	    let mut fm = f;
	    let _ = fm.write_all(&rb);
	    Ok(target_id)
	}
	Err(e) => Err(format!("Error: {}", e)),
    }
}

fn delete(target_id: &str) -> Result<&str, String> {
    let mut result: String = format!("id,name,status,description\n");

    for line in read_to_string(TODO_PATH).unwrap().lines() {
	let items: Vec<&str> = line.split(",").collect();
	if items.len() != 4 {
	    continue;
	}
	let id: String = items[0].to_string();
	let line: String = match target_id == id {
	    true => continue,
	    _ => {
		let stt: usize = items[2].parse().unwrap();
		let st: TodoStatus = TodoStatus::from_usize(stt).unwrap();
		let input = Todo {
		    id,
		    todo: TodoInput {
			name: items[1].to_string(),
			status: st,
			description: items[3].to_string(),
		    },
		};
		input.line()
	    }
	};
	result = format!("{}\n{}", result, line);
    }
    let rb = result.as_bytes();
    match File::open(TODO_PATH) {
	Ok(f) => {
	    let mut fm = f;
	    let _ = fm.write_all(&rb);
	    Ok(target_id)
	}
	Err(e) => Err(format!("error: {}", e)),
    }
}
fn list() -> Option<Vec<Todo>> {
    let mut result = Vec::new();

    for line in read_to_string(TODO_PATH).unwrap().lines() {
	let items: Vec<&str> = line.split(",").collect();
	if items.len() != 4 {
	    return None;
	}
	let id: String = items[0].parse().unwrap();
	let stt: usize = items[2].parse().unwrap();
	let st: TodoStatus = TodoStatus::from_usize(stt).unwrap();
	let t = Todo {
	    id,
	    todo: TodoInput {
		name: items[1].to_string(),
		description: items[3].to_string(),
		status: st as TodoStatus,
	    },
	};
	result.push(t)
    }

    Some(result)
}
fn help() {
    let help_list = [
	("create", "<bin> create name status descrption"),
	("list", "<bin> list"),
	("update", "<bin> update id name status description"),
	("delete", "<bin> id"),
    ];
    for (name, desc) in help_list {
	println!("{}, {}", name, desc);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = args[0].as_str();
    match cmd {
	"help" => help(),
	"create" => {
	    let name: String = args[2].parse().unwrap();
	    let sstatus: usize = args[3].parse().unwrap();
	    let status: TodoStatus = TodoStatus::from_usize(sstatus).unwrap();
	    let description: String = args[4].parse().unwrap();
	    let rst = create(TodoInput {
		name,
		status,
		description,
	    });

	    match rst {
		Ok(iid) => println!("{} created.", iid),
		Err(e) => eprintln!("{} create failed.", e),
	    }
	}
	"list" => {
	    let list: Option<Vec<Todo>> = list();
	    match list {
		Some(l) => {
		    for x in l {
			println!(
			    "id: {}, name: {}, status: {}, \n\tdescription: {}",
			    x.id, x.todo.name, x.todo.status, x.todo.description
			);
		    }
		    return ();
		}
		None => (),
	    }
	}
	"update" => {
	    let id: String = args[1].parse().unwrap();
	    let name: String = args[2].parse().unwrap();
	    let sstatus: usize = args[3].parse().unwrap();
	    let status: TodoStatus = TodoStatus::from_usize(sstatus).unwrap();
	    let description: String = args[4].parse().unwrap();
	    let rst = update(
		&id,
		TodoUpdateInput {
		    name: Some(name),
		    status: Some(status),
		    description: Some(description),
		},
	    );
	    match rst {
		Ok(iid) => println!("{} updated.", iid),
		Err(e) => eprintln!("{} update failed.", e),
	    }
	}
	"delete" => {
	    let id: String = args[1].parse().unwrap();
	    match delete(id.as_str()) {
		Ok(iid) => println!("{} deleted.", iid),
		Err(e) => eprintln!("{} delete failed.", e),
	    }
	}
	_ => (),
    }
}

fn detail(id: &str) -> Option<Todo> {
    match list() {
	Some(l) => {
	    for t in l {
		if t.id == id {
		    return Some(t);
		}
	    }
	    None
	}
	None => None,
    }
}

fn change_status(id: &str, status: TodoStatus) -> Result<&str, String> {
    update(
	id,
	TodoUpdateInput {
	    name: None,
	    description: None,
	    status: Some(status),
	},
    )
}

fn change_doing_status(id: &str) -> Result<&str, String> {
    change_status(id, TodoStatus::Doing)
}
fn change_doit_status(id: &str) -> Result<&str, String> {
    change_status(id, TodoStatus::Doit)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::*;
    fn cleanup() {
	// remove file.
	let rst = fs::remove_file(TODO_PATH);
	match rst {
	    Ok(_) => (),
	    Err(_e) => panic!(),
	}
    }
    #[test]
    fn list_todo() {}
    #[test]
    fn create_todo() {
	cleanup();
	let t = TodoInput {
	    name: String::from("test"),
	    description: String::from("koreha test"),
	    status: TodoStatus::Todo,
	};
	// 実処理
	match create(t) {
	    Ok(id) => {
		// todoファイル存在確認
		let exists = std::path::Path::new(TODO_PATH).exists();
		let contents = read_to_string(TODO_PATH).unwrap();
		let expect = format!(
		    "id,name,status,description\n{},{},{},{}",
		    id,
		    "test",
		    TodoStatus::Todo,
		    "koreha test",
		);
		assert_eq!(contents, expect);
		assert!(exists)
	    }
	    Err(e) => {
		eprintln!("{}", e);
		assert!(false)
	    }
	}
	cleanup();
    }
    #[test]
    fn update_todo() {
	cleanup();
	let t = TodoInput {
	    name: String::from("test"),
	    description: String::from("koreha test"),
	    status: TodoStatus::Todo,
	};
	// 実処理
	let id = match create(t) {
	    Ok(i) => i,
	    Err(_e) => "".to_string(),
	};
	let ut = TodoUpdateInput {
	    name: None,
	    description: None,
	    status: Some(TodoStatus::Doit),
	};
	let _ = update(&id, ut);
	// 対象データが更新されていること
	let act_line = match detail(&id) {
	    Some(at) => {
		format!(
		    "{},{},{},{}",
		    at.id, at.todo.name, at.todo.status, at.todo.description
		)
	    }
	    None => {
		let at = Todo {
		    id: "".to_string(),
		    todo: TodoInput {
			name: "".to_string(),
			description: "".to_string(),
			status: TodoStatus::Doing,
		    },
		};
		format!(
		    "{},{},{},{}",
		    at.id, at.todo.name, at.todo.status, at.todo.description
		)
	    }
	};
	let ext = Todo {
	    id,
	    todo: TodoInput {
		name: String::from("test"),
		description: String::from("koreha test"),
		status: TodoStatus::Todo,
	    },
	};
	let ext_line = format!(
	    "{},{},{},{}",
	    ext.id, ext.todo.name, ext.todo.status, ext.todo.description
	);
	assert_eq!(ext_line, act_line);
	cleanup();
    }
}
