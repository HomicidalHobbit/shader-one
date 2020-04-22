use std::fs;
use std::str::Lines;

#[derive(Copy, Clone)]
enum Stages {
    Vertex,
    Fragment,
    StageCount,
}

struct PassStages {
    stages: Vec<Stages>,
    code: Vec<String>,
}

impl PassStages {
    fn new() -> Self {
        PassStages {
            stages: Vec::with_capacity(Stages::StageCount as usize),
            code: Vec::with_capacity(Stages::StageCount as usize),
        }
    }
}

pub struct Parser {
    file: String,
    name: Option<String>,
    shader: Option<String>,
    pass_names: Vec<String>,
    pass_start: Vec<usize>,
    pass_end: Vec<usize>,
    pass_stages: PassStages,
}

impl Parser {
    pub fn new(name: &str) -> Self {
        let mut p = Parser {
            file: name.to_string(),
            name: None,
            shader: Some(fs::read_to_string(name).unwrap()),
            pass_names: Vec::new(),
            pass_start: Vec::new(),
            pass_end: Vec::new(),
            pass_stages: PassStages::new(),
        };
        p.get_name();
        p.get_passes();
        p
    }
}

impl Parser {
    pub fn get_name(&mut self) -> &Option<String> {
        if let Some(shader) = &self.shader {
            let mut lines = shader.lines();
            while let Some(line) = lines.next() {
                let v: Vec<&str> = line.trim().split(' ').collect();
                if !v.is_empty() && v[0].to_uppercase() == "NAME" {
                    let n: Vec<&str> = line.split('"').collect();
                    self.name = Some(n[1].to_string());
                    println!("Shader '{}'", n[1]);
                    return &self.name;
                }
            }
        }
        &self.name
    }

    pub fn get_passes(&mut self) {
        if let Some(shader) = &self.shader {
            let mut count = 0;
            let mut lines = shader.lines();
            while let Some(mut line) = lines.next() {
                count += 1;
                let v: Vec<&str> = line.trim().split(' ').collect();
                if !v.is_empty() && v[0].to_uppercase() == "PASS" {
                    let n: Vec<&str> = line.split('"').collect();
                    self.pass_names.push(n[1].to_string());
                    println!(
                        "Found Pass '{}'",
                        self.pass_names[self.pass_names.len() - 1]
                    );

                    let mut stages: [(Option<Stages>, usize, usize, usize);
                        Stages::StageCount as usize] = Default::default();
                    let mut indent = 0;
                    let mut found = false;
                    let mut first = true;
                    let mut start = count;
                    let mut end = count;
                    let mut doing_stage: Option<Stages> = None;
                    let mut end_stage: Option<Stages> = None;

                    loop {
                        let a: Vec<&str> = line.trim().split('{').collect();
                        for s in a {
                            if s.is_empty() {
                                if first == true {
                                    first = false;
                                    start = count;
                                }
                                indent += 1;
                            }
                        }
                        let b: Vec<&str> = line.trim().split('}').collect();
                        for s in b {
                            if s.is_empty() {
                                if indent == 0 {
                                    found = true;
                                    end = count;
                                    break;
                                }
                                indent -= 1;
                            }
                        }

                        count += 1;
                        let next_line = lines.next();
                        if next_line == None {
                            end = count; // ERROR unintended end
                            break;
                        }

                        if found {
                            if let Some(s) = doing_stage {
                                stages[s as usize].2 = count - 2;
                            }
                            break;
                        }

                        line = next_line.unwrap();
                        let upper = line.trim().to_uppercase();

                        if upper.ends_with("[VERT]") || upper.ends_with("[VERTEX]") {
                            stages[Stages::Vertex as usize] =
                                (Some(Stages::Vertex), count + 1, 0, 0);
                            end_stage = doing_stage;
                            doing_stage = Some(Stages::Vertex);
                        }

                        if upper.ends_with("[FRAG]") || upper.ends_with("[FRAGMENT]") {
                            stages[Stages::Fragment as usize] =
                                (Some(Stages::Fragment), count + 1, 0, 0);
                            end_stage = doing_stage;
                            doing_stage = Some(Stages::Fragment);
                        }

                        if upper.ends_with("ENTRY") {
                            if let Some(s) = doing_stage {
                                stages[s as usize].3 = count;
                            }
                        }

                        if upper.starts_with("VARIANTS") {
                            println!("Found Variant Statement!");
                        }

                        if let Some(s) = end_stage {
                            stages[s as usize].2 = count - 1;
                            end_stage = None;
                        }
                    }

                    println!("Start: {}", start);
                    println!("End: {}", end);
                    self.pass_start.push(start);
                    self.pass_end.push(end);

                    for stage in &stages {
                        if let Some(s) = stage.0 {
                            match s {
                                Stages::Vertex => print!("vertex shader:"),
                                Stages::Fragment => print!("fragment shader:"),
                                _ => print!("unknown shader stage:"),
                            }
                            println!(" {} to {}, entry: {}", stage.1, stage.2, stage.3);
                        }
                    }
                }
            }
        }
    }

    fn goto(line: usize, lines: &mut Lines) {
        let mut count = 0;
        while count < line {
            count += 1;
            lines.next();
        }
    }
}
