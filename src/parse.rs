#![feature(proc_macro)]

extern crate either;
extern crate proc_macro;
extern crate proc_macro2;

use syn::Path;
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Expr, Ident, LitBool, LitInt, Type};
use std::convert::From;
use std::iter::FromIterator;
use error::*;

use either::Either;
use proc_macro2::TokenStream;

use syn;
use {App, Init, Resources};

struct Fail {}

impl Synom for Fail {
    named!(parse -> Self, do_parse!(
        _fail: syn!(Type) >>
        (Fail {})
    ));
}

macro_rules! key {
    // `()` indicates that the macro takes no argument.
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

// #[derive(Debug)]
// struct App {
//     device: Option<Path>,
//     resources: Vec<ResFields>,
//     init: Option<Path>,
//     idle_path: Option<Path>,
//     idle_resources: Option<Vec<
//     tasks: Vec<(Ident, Vec<EnumTask>)>,
// }

pub fn parse_app(input: proc_macro::TokenStream) -> Result<App> {
    let app: Punct<AppFields, Token![,]> = syn::parse(input).chain_err(|| "parsing `app`")?;

    let mut device: Option<Path> = None;
    let mut resources: Vec<ResFields> = Vec::new();
    let mut init: Option<Init> = None;
    let mut idle: Vec<IdleFields> = Vec::new();
    let mut tasks: Vec<(Ident, Vec<EnumTask>)> = Vec::new();

    //let mut ok = true;

    for AppFields { key, value } in app.data.into_iter() {
        //println!("k {:?} v {:?}", key, value)
        match key.as_ref() {
            "device" => match value.as_ref().left() {
                Some(path) => {
                    if device == None {
                        device = Some(path.clone());
                    } else {
                        bail!("Field `device` multiple defined.");
                    }
                }
                _ => {
                    println!("device should be a path");
                    panic!("internal error");
                }
            },
            // "resources" => {
            //     println!("resources");
            //     if resources.is_empty() {
            //         match value.right() {
            //             Some(ts) => {
            //                 println!("ts");
            //                 let res: Punct<ResFields, Token![;]> = syn::parse2(ts).unwrap();
            //                 resources = Vec::from_iter(res.data.into_iter());
            //             }
            //             _ => {
            //                 println!("expected list of resource definitions");
            //                 panic!("internal error");
            //             }
            //         }
            //     } else {
            //         println!("Field `resource` multiple defined.");
            //         ok = false;
            //     }
            // }
            "init" => {
                println!("init");
                if init == None {
                    match value.right() {
                        Some(ts) => {
                            println!("ts");

                            let mut init_path: Option<Path> = None;
                            let mut resources = None;
                            let pt: Punct<EnumIdle, Token![,]> = syn::parse2(ts).unwrap();
                            for e in pt.data.into_iter() {
                                match e {
                                    EnumIdle::IdlePath(path) => {
                                        if init_path == None {
                                            println!("path {:?}", path);
                                            init_path = Some(path);
                                        } else {
                                            bail!("Field `device` multiple defined.");
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
                            init = Some(Init {
                                path: init_path,
                                resources,
                                _extensible: (),
                            })
                        }
                        _ => {
                            println!("expected list of resource definitions");
                            panic!("internal error");
                        }
                    }
                } else {
                    bail!("Field `init` multiple defined.");
                }
            }
            // "idle" => {
            //     println!("idle");
            //     if idle.is_empty() {
            //         match value.right() {
            //             Some(ts) => {
            //                 println!("ts");

            //                 let idle: Punct<EnumIdle, Token![,]> = syn::parse2(ts).unwrap();
            //                 for e in idle.data.iter() {
            //                     match e {
            //                         EnumIdle::IdlePath(path) => println!("path {:?}", path),
            //                         EnumIdle::IdleResources(res) => println!("resources {:?}", res),
            //                     }
            //                 }
            //             }
            //             _ => {
            //                 println!("expected list of idle definitions");
            //                 panic!("internal error");
            //             }
            //         }
            //     } else {
            //         println!("Field `idle` multiple defined.");
            //         ok = false;
            //     }
            // }
            // "tasks" => {
            //     println!("tasks");
            //     if tasks.is_empty() {
            //         match value.right() {
            //             Some(ts) => {
            //                 println!("ts");

            //                 let tasks: Punct<Tasks, Token![,]> = syn::parse2(ts).unwrap();

            //                 for Tasks { id, task } in tasks.data.into_iter() {
            //                     println!("task {}", id.as_ref());

            //                     let task: Punct<
            //                         EnumTask,
            //                         Token![,],
            //                     > = syn::parse2(task).unwrap();
            //                     for tf in task.data {
            //                         match tf {
            //                             EnumTask::TaskPrio(prio) => println!("prio"),
            //                             EnumTask::TaskPath(path) => println!("path"),
            //                             EnumTask::TaskResources(res) => println!("res"),
            //                             EnumTask::TaskEnabled(b) => println!("bool"),
            //                         }
            //                     }
            //                 }
            //             }
            //             _ => {
            //                 println!("expected list of task definitions");
            //                 panic!("internal error");
            //             }
            //         }
            //     } else {
            //         println!("Field `tasks` multiple defined.");
            //         ok = false;
            //     }
            // }
            _ => {
                bail!("Illegal field {}.", key.as_ref());
            }
        }
    }

    if let Some(device) = device {
        Ok(App {
            device,
            init,
            // idle,
            // resources,
            // tasks,
        })
    } else {
        bail!("Field `device` missing.");
    }
}

// Vec[T] (or perhaps not quite)
struct Punct<T, P>
where
    T: Synom,
    P: Synom,
{
    data: Punctuated<T, P>,
}

// Parse a comma separated TokenStream into a Vec[T] (or perhaps not quite)
impl<T, P> Synom for Punct<T, P>
where
    T: Synom,
    P: Synom,
{
    named!(parse -> Self, map!(call!(Punctuated::parse_terminated_nonempty), |data| Punct { data }));
}

// Parse the top level app!
// { device : Path, resources: {}, init: {}, ... }
struct AppFields {
    key: Ident,
    value: Either<Path, TokenStream>,
}

impl Synom for AppFields {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
        _colon: punct!(:) >>
        value: alt!(
            map!(syn!(Path), |path| Either::Left(path)) |
            map!(braces!(syn!(TokenStream)), |(_, ts)| Either::Right(ts))
        ) >>
        (AppFields { key, value })
    ));
}

#[derive(Debug)]
struct ResFields {
    ident: Ident,
    type_: Type,
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
        type_: syn!(Type) >>
        expr: option!(
            do_parse!(
                _eq: syn!(Token![=]) >>
                expr: syn!(Expr) >>
                (expr)
            )
        ) >>
        (ResFields { ident, type_, expr })
    ));
}

struct PathField {
    path: Path,
}

// Parse the init
// path: main::init
impl Synom for PathField {
    named!(parse -> Self, do_parse!(
        _key: syn!(KeyPath) >>
        _colon: syn!(Token![:]) >>
        path: syn!(Path) >>
        (PathField { path })
    ));
}

struct IdleFields {
    key: Ident,
    value: TokenStream,
}

impl Synom for IdleFields {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
        _colon: punct!(:) >>
        value: syn!(TokenStream) >>
        (IdleFields { key, value })
    ));
}

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

struct Tasks {
    id: Ident,
    task: TokenStream,
}

impl Synom for Tasks {
    named!(parse -> Self, do_parse!(
        id: syn!(Ident) >>
        _colon: punct!(:) >>
        task: braces!(syn!(TokenStream)) >>
        (Tasks{id, task: task.1})
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
