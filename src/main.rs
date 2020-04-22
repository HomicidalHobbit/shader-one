/*
Author: Richard Underhill 2020
*/

#![allow(dead_code)]
mod freetype;
mod glfw;
mod shader;
mod vulkan;

use core::ptr::null;
use shader::Parser;
use std::ffi::{CStr, CString};
use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::os::raw::c_char;
use std::ptr::copy;

use core::ffi::c_void;
use glfw::{glfwInit, glfwTerminate, glfwVulkanSupported};
const GLFW_TRUE: i32 = 1;

enum Stage {
    VertexStage,
    TessControlStage,
    TessEvaluationStage,
    GeometryStage,
    FragmentStage,
    ComputeStage,
}

enum GraphicsAPI {
    GLSL,
    HLSL,
    Metal,
}

extern "C" {

    fn Initialise();
    fn GetSpirvSize() -> usize;
    fn CreateProgram() -> *const c_void;
    fn DeleteProgram(program: *const c_void);
    fn CompileShader(stage: i32, source: *const c_char) -> usize;
    fn GetShader(handle: usize) -> *const c_void;
    fn Add(program: *const c_void, handle: usize);
    fn AddKeyword(program: *const c_char) -> u64;
    fn GetKeywordsID() -> u64;
    fn GetKeywordsFromID(id: u64) -> *const c_char;
    fn ReserveKeyword(keyword: *const c_char) -> u64;
    fn EnableKeyword(keyword: *const c_char) -> bool;
    fn EnableGlobalKeyword(keyword: *const c_char) -> bool;
    fn DisableKeyword(keyword: *const c_char) -> bool;
    fn DisableGlobalKeyword(keyword: *const c_char) -> bool;
    fn PrintKeywords() -> *const c_char;
    fn ClearPreamble();
    fn SetPreamble(source: *const c_char);
    fn PrintSpirv();
    fn Recompile(handle: usize) -> usize;
    fn DecompileToGLSL() -> *const c_char;
    fn DecompileToHLSL() -> *const c_char;
    fn DecompileToMetal() -> *const c_char;
    fn Link(program: *const c_void) -> bool;
    fn GetSpirvForStage(program: *const c_void, stage: i32) -> *const c_void;
    fn Disassemble(spirv: *const u32, length: usize) -> *const c_char;
    fn ClearShaderCache();
    fn CreateBook();
    fn AddChapter();
    fn Shutdown();
}

struct Variants {
    kws: Vec<Vec<String>>,
}

impl Variants {
    fn new() -> Self {
        Variants { kws: Vec::new() }
    }
}

impl Variants {
    fn push(&mut self, layer: Vec<String>) {
        self.kws.push(layer);
    }
}

struct Program {
    program: *const c_void,
    keywords: String,
    linked: bool,
}

struct ShaderCompiler {
    spirv: Vec<u32>,
    source: String,
    csource: Option<CString>,
    programs: Vec<Program>,
    programs_free_list: Vec<usize>,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        unsafe {
            Initialise();
        }
        ShaderCompiler {
            spirv: Vec::new(),
            source: String::new(),
            csource: None,
            programs: Vec::new(),
            programs_free_list: Vec::new(),
        }
    }

    pub fn reserve_keyword(kw: &str) {
        let cs = CString::new(kw).unwrap();
        let csource: *const c_char = cs.as_ref().as_ptr() as *const c_char;
        unsafe {
            if !ReserveKeyword(csource) == 0 {
                panic!("Unable to add keyword '{}'", kw);
            }
        }
    }

    pub fn enable_global_keyword(kw: &str) {
        let cs = CString::new(kw).unwrap();
        let csource: *const c_char = cs.as_ref().as_ptr() as *const c_char;
        unsafe {
            if !EnableGlobalKeyword(csource) {
                panic!("Unable to enable keyword '{}'", kw);
            }
        }
    }

    pub fn disable_global_keyword(kw: &str) {
        let cs = CString::new(kw).unwrap();
        let csource: *const c_char = cs.as_ref().as_ptr() as *const c_char;
        unsafe {
            if !DisableGlobalKeyword(csource) {
                panic!("Unable to disable keyword '{}'", kw);
            }
        }
    }

    pub fn load_shader(name: &str) -> String {
        fs::read_to_string(name).unwrap()
    }
}

impl ShaderCompiler {
    pub fn create_program(&mut self) -> usize {
        let mut handle: usize;
        let program: *const c_void;
        unsafe {
            program = CreateProgram();
        }

        if self.programs_free_list.is_empty() {
            self.programs.push(Program {
                program,
                keywords: String::new(),
                linked: false,
            });
            handle = self.programs.len();
        } else {
            handle = self.programs_free_list[0];
            if let Some(p) = self.programs_free_list.pop() {
                self.programs_free_list[0] = p;
                handle = p + 1;
            }
        }
        handle
    }

    pub fn delete_program(&mut self, handle: usize) {
        let index = handle - 1;
        unsafe {
            DeleteProgram(self.programs[index].program);
        }
        self.programs[index].program = null();
        self.programs_free_list.push(index);
    }

    pub fn clear_shader_cache(&self) {
        unsafe {
            ClearShaderCache();
        }
    }

    pub fn compile(&mut self, stage: Stage, source: &str) -> usize {
        self.csource = Some(CString::new(source).unwrap());
        let csource: *const c_char = self.csource.as_ref().unwrap().as_ptr() as *const c_char;
        let handle: usize;
        unsafe {
            handle = CompileShader(stage as i32, csource);
        }
        handle
    }

    pub fn compile_from_file(&mut self, stage: Stage, name: &str) -> usize {
        println!("Reading File: '{}'", name);
        match read_to_string(name) {
            Ok(f) => self.compile(stage, &f),
            Err(_) => {
                println!("Error: cannot read file '{}'!", name);
                0
            }
        }
    }

    pub fn enable_keyword(&self, kw: &str) {
        let cs = CString::new(kw).unwrap();
        let csource: *const c_char = cs.as_ref().as_ptr() as *const c_char;
        unsafe {
            if !EnableKeyword(csource) {
                panic!("Unable to enable keyword '{}'", kw);
            }
        }
    }

    pub fn disable_keyword(&self, kw: &str) {
        let cs = CString::new(kw).unwrap();
        let csource: *const c_char = cs.as_ref().as_ptr() as *const c_char;
        unsafe {
            if !DisableKeyword(csource) {
                panic!("Unable to disable keyword '{}'", kw);
            }
        }
    }

    pub fn get_keywords_id(&self) -> u64 {
        unsafe { GetKeywordsID() }
    }

    pub fn add_keyword(&mut self, program: usize, kw: &str) {
        self.csource = Some(CString::new(kw).unwrap());
        let csource: *const c_char = self.csource.as_ref().unwrap().as_ptr() as *const c_char;
        unsafe {
            if !AddKeyword(csource) == 0 {
                panic!("Unable to add keyword '{}'", kw);
            }
        }
        let index = program - 1;
        self.programs[index].keywords.push_str(kw);
        self.programs[index].keywords.push('\n');
    }

    pub fn set_preamble(&mut self, source: &str) {
        self.csource = Some(CString::new(source).unwrap());
        let csource: *const c_char = self.csource.as_ref().unwrap().as_ptr() as *const c_char;
        unsafe {
            SetPreamble(csource);
        }
    }

    pub fn clear_preamble(&self) {
        unsafe {
            ClearPreamble();
        }
    }

    pub fn recompile(&self, handle: usize) -> usize {
        unsafe { Recompile(handle) }
    }

    pub fn get_spirv_for_stage(&mut self, program: usize, stage: Stage) {
        let index = program - 1;
        unsafe {
            let ptr = GetSpirvForStage(self.programs[index].program, stage as i32);
            if ptr != null() && self.programs[index].linked {
                let size = GetSpirvSize();
                self.spirv.resize_with(size, Default::default);
                copy(ptr as *mut u32, self.spirv.as_mut_ptr(), size);
            } else {
                self.spirv.clear();
            }
        }
    }

    pub fn get_spirv_size(&self) -> usize {
        self.spirv.len()
    }

    pub fn add(&mut self, program: usize, handle: usize) {
        unsafe {
            Add(self.programs[program - 1].program, handle);
        }
    }

    pub fn link(&mut self, program: usize) -> bool {
        let index = program - 1;
        unsafe {
            let linked = Link(self.programs[index].program);
            self.programs[index].linked = linked;
            linked
        }
    }

    pub fn save_spirv(&self, dest: &mut Vec<u32>) {
        *dest = self.spirv.clone()
    }

    pub fn write_spirv(&mut self, name: &str) {
        let mut file = File::create(name).unwrap();
        unsafe {
            let current_length = self.spirv.len();
            self.spirv
                .set_len(current_length * std::mem::size_of::<u32>());
            let slice: &[u8] = std::mem::transmute::<&[u32], &[u8]>(self.spirv.as_slice());
            file.write(slice).unwrap();
            self.spirv.set_len(current_length);
        }
    }

    pub fn read_spirv(&mut self, name: &str) {
        let meta = fs::metadata(name).unwrap();
        let size = meta.len() as usize;
        let mut file = File::open(name).unwrap();
        let u32size = size / std::mem::size_of::<u32>();
        unsafe {
            self.spirv.resize_with(u32size, Default::default);
            self.spirv.set_len(size);
            let slice: &mut [u8] =
                std::mem::transmute::<&mut [u32], &mut [u8]>(self.spirv.as_mut_slice());
            file.read(slice).unwrap();
            self.spirv.set_len(u32size);
        }
    }

    pub fn get_keywords_from_id(&mut self, id: u64) -> String {
        let kw: *const c_char;
        unsafe {
            kw = GetKeywordsFromID(id);
            let c_str: &CStr = CStr::from_ptr(kw);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.to_owned()
        }
    }

    pub fn disassemble_spirv(&mut self) -> String {
        let length = self.spirv.len();
        let source: *const c_char;
        unsafe {
            source = Disassemble(self.spirv.as_ptr(), length);
            let c_str: &CStr = CStr::from_ptr(source);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.to_owned()
        }
    }

    pub fn decompile_spirv(&self, gapi: GraphicsAPI) -> String {
        let source: *const c_char;
        unsafe {
            match gapi {
                GraphicsAPI::GLSL => source = DecompileToGLSL(),
                GraphicsAPI::HLSL => source = DecompileToHLSL(),
                GraphicsAPI::Metal => source = DecompileToMetal(),
            };
            let c_str: &CStr = CStr::from_ptr(source);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.to_owned()
        }
    }

    pub fn print_spirv(&self) {
        unsafe { PrintSpirv() }
    }

    pub fn print_keywords(&self) {
        unsafe {
            PrintKeywords();
        }
    }

    pub fn create_book(&self) {
        unsafe {
            CreateBook();
        }
    }

    pub fn add_chapter(&self) {
        unsafe {
            AddChapter();
        }
    }
}

impl Drop for ShaderCompiler {
    fn drop(&mut self) {
        unsafe {
            for program in &mut self.programs {
                if program.program != null() {
                    DeleteProgram(program.program);
                    program.program = null();
                }
            }
            Shutdown();
        }
    }
}

fn main() {
    println!("Hello, world!");
    unsafe {
        if glfwInit() == GLFW_TRUE {
            println!("GLFW Initialised OK!");

            if glfwVulkanSupported() == GLFW_TRUE {
                println!("Vulkan Supported!");
            }
        }
        glfwTerminate();

        let _shader = Parser::new("src/shaders/deferredwithtransparent.esl");

        let mut variants = Variants::new();
        variants.push(vec!["NOT_LIT".to_string(), "LIT".to_string()]);
        variants.push(vec!["NOT_LOD_BIAS".to_string(), "LOD_BIAS".to_string()]);

        create_variants(&variants);
    }
}

fn create_variants(variants: &Variants) {
    let mut compiler = ShaderCompiler::new();
    let p = compiler.create_program();

    let v = compiler.compile_from_file(Stage::VertexStage, "test.vert");
    let f = compiler.compile_from_file(Stage::FragmentStage, "test.frag");
    compiler.add(p, v);
    compiler.add(p, f);
    compiler.link(p);

    let rows = variants.kws.len();
    let mut row_indices: Vec<usize> = Vec::with_capacity(rows);

    for _ in 0..rows {
        row_indices.push(0);
    }

    let mut done = false;
    let mut count = 0;
    while !done {
        let np = compiler.create_program();
        for r in 0..rows {
            compiler.add_keyword(p, &variants.kws[r][row_indices[r]]);
        }
        let nv = compiler.recompile(v);
        let nf = compiler.recompile(f);

        compiler.add(np, nv);
        compiler.add(np, nf);
        compiler.link(np);

        println!(
            "Keywords:\n{}",
            compiler.get_keywords_from_id(compiler.get_keywords_id())
        );

        count += 1;
        for r in (0..rows).rev() {
            row_indices[r] += 1;
            if row_indices[r] != variants.kws[r].len() {
                break;
            }
            row_indices[r] = 0;
            if r == 0 {
                done = true;
                break;
            }
        }
    }
    compiler.print_keywords();
    println!("Variants count: {}", count);
}
