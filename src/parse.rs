#![feature(proc_macro)]

extern crate either;
extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;

//use error::Error;
use std::fmt::Debug;

use syn::Path;
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Expr, Ident, LitBool, LitInt, Type};
use std::convert::From;
// use std::iter::FromIterator;
use error::*;
use std::collections::{HashMap, HashSet};

use either::Either;
use proc_macro2::TokenStream;

use syn;
use {App, Idle, Init, Resources,    Statics, Task, Tasks};

struct Fail {}

    impl Synom for Fail {
    named!(parse -> Self, do_parse!(
        _fail: syn!(Type) >>
        (Fail {})
    ));
}

macro_rules! key {
    ($id: ident, $key: expr) => {
        #[derive(Debug)]
        struct $id {}

        impl Synom for $id {
            fn parse(i: syn::buffer::Cursor) -> syn::synom::PResult<Self> {
                match <Ident>::parse(i) {
                    ::std::result::Result::Err(err) => ::std::result::Result::Err(err),
                    ::std::result::Result::Ok((o, i)) => {
                        if o.as_ref() == $key {
                            ::std::result::Result::Ok((($id {}), i))
                        } else {
                            match <Fail>::parse(i) {
                                ::std::result::Result::Err(err) => {
                                    // let s = String::from(format!(
                                    //     "expected `{:?}`, got {:?}",
                                    //     $key,
                                    //     o.as_ref()
                                    // ));

                                    // o.span.unstable().warning(s).emit();

                                    ::std::result::Result::Err(err)
                                }
                                _ => panic!("internal error"),
                            }
                        }
                    }
                }
            }
        }
    };
}

key!{KeyDevice, "device"}
key!{KeyPath, "path"}
key!{KeyResources, "resources"}
key!{KeyPrio, "priority"}
key!{KeyEnabled, "enabled"}

fn check_dup<T>(key_val: &Punct<KeyValue<T>, Token![,]>) -> Result<()>
where
    T: Synom,
{
    let mut keys = HashSet::new();
    let mut err = false;
    for KeyValue { key, value: _ } in key_val.data.iter() {
        println!("key {:?}", key.as_ref());
        if !keys.insert(key) {
            let s = String::from(format!("Field `{:?}` multiple defined", key.as_ref()));

            key.span.unstable().error(s).emit();

            let first = keys.get(key).unwrap();
            first.span.unstable().warning("first defined here").emit();

            err = true;
        }
    }
    if err {
        bail!("Dupclicate(s) found.");
    } else {
        Ok(())
    }
}

fn error(key: &Ident, msg: &str) -> Result<App> {
    let s = String::from(msg);
    key.span.unstable().error(s).emit();
    bail!(msg);
}

use separated::Pun;

pub fn parse_app(input: proc_macro::TokenStream) -> Result<App> {
    println!("--------------------------------------- debug");
    let x: Pun<Ident, Token![,]> = syn::parse(input).unwrap();
    println!("x {:?}", x);
    panic!("<<< ok");





    let app: Punct<KeyValue<AppValue>, Token![,]> =
        syn::parse(input).chain_err(|| "parsing `app`")?;

    check_dup(&app)?;

    let mut device: Option<Path> = None;
    let mut resources: Option<Statics> = None;
    let mut init: Option<Init> = None;
    let mut idle: Option<Idle> = None;
    let mut tasks: Option<Tasks> = None;

    for KeyValue { key, value } in app.data.into_iter() {
        match key.as_ref() {
            "device" => match value.value.left() {
                Some(path) => {
                    device = Some(path);
                }
                _ => {
                    error(&key, "`device` path expected")?;
                }
            },
            "resources" => {
                println!("resources");

                match value.value.right() {
                    Some(ts) => {
                        println!("-- parse resources --");
                        let mut hm = Statics::new();

                        match syn::parse2::<ParseResult<Punct<ResFields, Token![;]>>>(ts) {
                            // ParseResult<Punct<ResFields, Token![;]>>
                            Ok(ParseResult{ res }) => {
                                println!("-- ok --");
                                panic!("happy path");

                            }
                            _ => {
                                println!("-- error --");
                                panic!("sad path");
                                bail!( "illegal `resource`");
                            },
                        }

                        // let res: Punct<ResFields, Token![;]> = syn::parse2(ts).chain_err(|| "illegal `resource`")?;
                        // for r in res.data.into_iter() {
                        //     hm.insert(
                        //         r.ident,
                        //         Static {
                        //             expr: r.expr,
                        //             ty: r.ty,
                        //             _extensible: (),
                        //         },
                        //     );
                        // }

                        resources = Some(hm);
                    }
                    _ => {
                        error(&key, "`resources` expected")?;
                    }
                }
            }
            "init" => {
                println!("init");

                match value.value.right() {
                    Some(ts) => {
                        let (path, resources) =
                            parse_path_resources(ts).chain_err(|| "parsing init")?;
                        init = Some(Init {
                            path,
                            resources,
                            _extensible: (),
                        })
                    }
                    _ => {
                        panic!("internal error");
                    }
                }
            }

            "idle" => {
                println!("idle");
                if idle == None {
                    match value.value.right() {
                        Some(ts) => {
                            let (path, resources) =
                                parse_path_resources(ts).chain_err(|| "parsing init")?;
                            idle = Some(Idle {
                                path,
                                resources,
                                _extensible: (),
                            })
                        }
                        _ => {
                            panic!("internal error");
                        }
                    }
                } else {
                    println!("Field `idle` multiple defined.");
                }
            }

            "tasks" => {
                println!("tasks");
                match value.value.right() {
                    Some(ts) => {
                        println!("ts");

                        let parse_tasks: Punct<TasksParse, Token![,]> = syn::parse2(ts).unwrap();

                        let mut hmt = HashMap::new();
                        for TasksParse { id, task } in parse_tasks.data.into_iter() {
                            println!("task {}", id.as_ref());
                            let mut t = Task {
                                enabled: None,
                                path: None,
                                priority: None,
                                interarrival: None,
                                resources: None,
                                _extensible: (),
                            };

                            let task: Punct<EnumTask, Token![,]> = syn::parse2(task).unwrap();
                            for tf in task.data {
                                match tf {
                                    EnumTask::TaskPrio(prio) => {
                                        t.priority = Some(prio.value() as u8)
                                    }
                                    EnumTask::TaskPath(path) => t.path = Some(path),
                                    EnumTask::TaskResources(res) => {
                                        let mut hs = Resources::new();
                                        for r in res.into_iter() {
                                            hs.insert(r);
                                        }
                                        t.resources = Some(hs);
                                    }
                                    EnumTask::TaskEnabled(b) => t.enabled = Some(b.value),
                                }
                            }
                            hmt.insert(id, t);
                        }
                        tasks = Some(hmt);
                    }
                    _ => {
                        println!("expected list of task definitions");
                        panic!("internal error");
                    }
                }
            }
            _ => {
                let s = String::from(format!(
                    "Illegal field `{}`, expected `device`, `resources`, `init`, `idle`, or `task`.",
                    key.as_ref()
                ));
                error(&key, &s)?;
            }
        }
    }

    if let Some(device) = device {
        Ok(App {
            device,
            init,
            idle,
            resources,
            tasks,
            _extensible: (),
        })
    } else {
        bail!("Field `device` missing.");
    }
}

fn parse_path_resources(ts: TokenStream) -> Result<(Option<Path>, Option<Resources>)> {
    let mut init_path: Option<Path> = None;
    let mut resources: Option<Resources> = None;
    let pt: Punct<EnumIdle, Token![,]> = syn::parse2(ts).unwrap();
    for e in pt.data.into_iter() {
        match e {
            EnumIdle::IdlePath(path) => {
                if init_path == None {
                    println!("path {:?}", path);
                    init_path = Some(path);
                } else {
                    bail!("Field `path` multiple defined.");
                }
            }
            EnumIdle::IdleResources(res) => {
                println!("resources {:?}", res);
                if resources != None {
                    bail!("Field 'resources` multiple defined.");
                } else {
                    let mut hs = Resources::new();
                    for r in res.into_iter() {
                        hs.insert(r);
                    }
                    resources = Some(hs);
                }
            }
        }
    }
    Ok((init_path, resources))
    }




// Vec[T] (or perhaps not quite)
#[derive(Debug)]
struct Punct<T, P>
where
    T: Synom,
    P: Synom,
{
    data: Punctuated<T, P>,
}

// Parse a comma separated TokenStream into a Vec[T] (or perhaps not quite)
// impl<T, P> Synom for Punct<T, P>
// where
//     T: Synom,
//     P: Synom,
// {
//     named!(parse -> Self, 
//     alt!(
//         map!(call!(Punctuated::parse_terminated_nonempty), |data| Punct { data })
//         |
//         map!(syn(TokenStream), |ts| {panic!("Punct Err {:?}", ts);})
//     );

// }

impl<T, P> Synom for Punct<T, P>
where
    T: Synom,
    P: Synom,
{
    named!(parse -> Self, 
        map!(call!(Punctuated::parse_terminated_nonempty), |data| Punct { data })
    );

}

// Parse the top level app!
// { device : Path, resources: {}, init: {}, ... }

struct KeyValue<T> {
    key: Ident,
    value: T,
}

impl<T> Synom for KeyValue<T>
where
    T: Synom,
{
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
        _colon: punct!(:) >>
        value: syn!(T)
         >>
        (KeyValue { key, value })
    ));
}

struct AppValue {
    value: Either<Path, TokenStream>,
}
impl Synom for AppValue {
    named!(parse -> Self, do_parse!(
    
        value: alt!(
            map!(syn!(Path), |path| Either::Left(path)) |
            map!(braces!(syn!(TokenStream)), |(_, ts)| Either::Right(ts))
        ) >>
        (AppValue { value })
    ));
}

// struct AppFields {
//     key: Ident,
//     value: Either<Path, TokenStream>,
// }

// impl Synom for AppFields {
//     named!(parse -> Self, do_parse!(
//         key: syn!(Ident) >>
//         _colon: punct!(:) >>
//         value: alt!(
//             map!(syn!(Path), |path| Either::Left(path)) |
//             map!(braces!(syn!(TokenStream)), |(_, ts)| Either::Right(ts))
//         ) >>
//         (AppFields { key, value })
//     ));
// }

#[derive(Debug)]
struct ResFields {
    ident: Ident,
    ty: Type,
    expr: Option<Expr>,
}

// Parse a resource, e.g.,
// static X : u32 = 3 + 3
// static Y : u32 // late resource
impl Synom for ResFields {
    named!(parse -> Self, do_parse!(
        _static: keyword!(static) >>
        ident: syn!(Ident) >>
        _colon: syn!(Token![:]) >>
        ty: syn!(Type) >>
        expr: option!(
            do_parse!(
                _eq: syn!(Token![=]) >>
                expr: syn!(Expr) >>
                (expr)
            )
        ) >>
        (ResFields { ident, ty, expr })
    ));
}

#[derive(Debug)]
struct ParseResult<T>
where
    T: Synom,
{
    res: Result<T>,
}

// Parse a field

impl<T> Synom for ParseResult<T>
where
    T: Synom + Debug,
{
    named!(parse -> Self, do_parse!(
        res: alt!(
            do_parse!(
                res: map!(syn!(T), |ts| {
                    println!("<< ParseResult, Ok {:?}>>", ts); ts 
                }) >>
                (Ok(res))
            )
            |
            do_parse!(
// named!(parse -> Self, map!(call!(Punctuated::parse_terminated_nonempty), |data| Punct { data }));

                res: map!(syn!(TokenStream), |ts| { 
                    println!("<< ParseResult, Err {:?}>>", ts);
                    panic!("ueothnth");
                }) >>
                
                // (Err(error::Error))
                // let s = String::from(format!(
                                    //     "expected `{:?}`, got {:?}",
                                    //     $key,
                                    //     o.as_ref()
                                    // ));

                                    // o.span.unstable().warning(s).emit();

                ( Err("foo error!".into()) )                
            )
        ) >>
        (ParseResult { res })
    ));
}

// struct PathField {
//     path: Path,
// }

// // Parse the init
// // path: main::init
// impl Synom for PathField {
//     named!(parse -> Self, do_parse!(
//         _key: syn!(KeyPath) >>
//         _colon: syn!(Token![:]) >>
//         path: syn!(Path) >>
//         (PathField { path })
//     ));
// }

// struct IdleFields {
//     key: Ident,
//     value: TokenStream,
// }

// impl Synom for IdleFields {
//     named!(parse -> Self, do_parse!(
//         key: syn!(Ident) >>
//         _colon: punct!(:) >>
//         value: syn!(TokenStream) >>
//         (IdleFields { key, value })
//     ));
// }

#[derive(Debug)]
enum EnumIdle {
    IdlePath(Path),
    IdleResources(Punctuated<Ident, Token![,]>),
}

// Parse the idle
// resources: [OWNED, SHARED]
// path: main::idle,
impl Synom for EnumIdle {
    named!(parse -> Self, 
        alt!(
            do_parse!(
                _path: syn!(KeyPath) >>
                _colon: punct!(:) >>
                path: syn!(Path) >>
                (EnumIdle::IdlePath(path))
            )
            | 
            do_parse!(
                _res: syn!(KeyResources) >>
                _colon: punct!(:) >>
                res: brackets!(
                    call!(Punctuated::<Ident, Token![,]>::parse_separated_nonempty)
                ) >>
                (EnumIdle::IdleResources(res.1))
            )
            // | TODO, error handling
        )  
    );
}

enum EnumTask {
    TaskPath(Path),
    TaskResources(Punctuated<Ident, Token![,]>),
    TaskPrio(LitInt),
    TaskEnabled(LitBool),
}
// path: main::idle,
impl Synom for EnumTask {
    named!(parse -> Self, 
        alt!(
            do_parse!(
                _path: syn!(KeyPath) >>
                _colon: punct!(:) >>
                path: syn!(Path) >>
                (EnumTask::TaskPath(path))
            )
            | 
            do_parse!(
                _res: syn!(KeyResources) >>
                _colon: punct!(:) >>
                res: brackets!(
                    call!(Punctuated::<Ident, Token![,]>::parse_separated_nonempty)
                ) >>
                (EnumTask::TaskResources(res.1))
            )
            | 
            do_parse!(
                _res: syn!(KeyPrio) >>
                _colon: punct!(:) >>
                prio: syn!(LitInt) >>
                (EnumTask::TaskPrio(prio))
            )
            | 
            do_parse!(
                _res: syn!(KeyEnabled) >>
                _colon: punct!(:) >>
                enabled: syn!(LitBool) >>
                (EnumTask::TaskEnabled(enabled))
            )
            // | TODO, error handling
        )  
    );
}

struct TasksParse {
    id: Ident,
    task: TokenStream,
}

impl Synom for TasksParse {
    named!(parse -> Self, do_parse!(
        id: syn!(Ident) >>
        _colon: punct!(:) >>
        task: braces!(syn!(TokenStream)) >>
        (TasksParse{id, task: task.1})
    ));
}

// fn check_app(app: App) -> Option<App> {
//     let mut ok = true;

//     // check device
//     if app.device == None {
//         println!("Field `device` missing.");
//         ok = false;
//     }

//     // check resources
//     for i in app.resources.iter() {
//         println!(" {:?}", i);
//     }
//     // check idle

//     if ok {
//         Some(app)
//     } else {
//         None
//     }
// }
