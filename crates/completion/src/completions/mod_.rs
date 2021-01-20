//! Completes mod declarations.

use std::iter;

use hir::{Module, ModuleSource};
use ide_db::{
    base_db::{SourceDatabaseExt, VfsPath},
    RootDatabase, SymbolKind,
};
use rustc_hash::FxHashSet;

use crate::CompletionItem;

use crate::{context::CompletionContext, item::CompletionKind, Completions};

/// Complete mod declaration, i.e. `mod $0 ;`
pub(crate) fn complete_mod(acc: &mut Completions, ctx: &CompletionContext) -> Option<()> {
    let mod_under_caret = match &ctx.mod_declaration_under_caret {
        Some(mod_under_caret) if mod_under_caret.item_list().is_none() => mod_under_caret,
        _ => return None,
    };

    let _p = profile::span("completion::complete_mod");

    let current_module = ctx.scope.module()?;

    let module_definition_file =
        current_module.definition_source(ctx.db).file_id.original_file(ctx.db);
    let source_root = ctx.db.source_root(ctx.db.file_source_root(module_definition_file));
    let directory_to_look_for_submodules = directory_to_look_for_submodules(
        current_module,
        ctx.db,
        source_root.path_for_file(&module_definition_file)?,
    )?;

    let existing_mod_declarations = current_module
        .children(ctx.db)
        .filter_map(|module| Some(module.name(ctx.db)?.to_string()))
        .collect::<FxHashSet<_>>();

    let module_declaration_file =
        current_module.declaration_source(ctx.db).map(|module_declaration_source_file| {
            module_declaration_source_file.file_id.original_file(ctx.db)
        });

    source_root
        .iter()
        .filter(|submodule_candidate_file| submodule_candidate_file != &module_definition_file)
        .filter(|submodule_candidate_file| {
            Some(submodule_candidate_file) != module_declaration_file.as_ref()
        })
        .filter_map(|submodule_file| {
            let submodule_path = source_root.path_for_file(&submodule_file)?;
            let directory_with_submodule = submodule_path.parent()?;
            let (name, ext) = submodule_path.name_and_extension()?;
            if ext != Some("rs") {
                return None;
            }
            match name {
                "lib" | "main" => None,
                "mod" => {
                    if directory_with_submodule.parent()? == directory_to_look_for_submodules {
                        match directory_with_submodule.name_and_extension()? {
                            (directory_name, None) => Some(directory_name.to_owned()),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                file_name if directory_with_submodule == directory_to_look_for_submodules => {
                    Some(file_name.to_owned())
                }
                _ => None,
            }
        })
        .filter(|name| !existing_mod_declarations.contains(name))
        .for_each(|submodule_name| {
            let mut label = submodule_name;
            if mod_under_caret.semicolon_token().is_none() {
                label.push(';');
            }
            CompletionItem::new(CompletionKind::Magic, ctx.source_range(), &label)
                .kind(SymbolKind::Module)
                .add_to(acc)
        });

    Some(())
}

fn directory_to_look_for_submodules(
    module: Module,
    db: &RootDatabase,
    module_file_path: &VfsPath,
) -> Option<VfsPath> {
    let directory_with_module_path = module_file_path.parent()?;
    let (name, ext) = module_file_path.name_and_extension()?;
    if ext != Some("rs") {
        return None;
    }
    let base_directory = match name {
        "mod" | "lib" | "main" => Some(directory_with_module_path),
        regular_rust_file_name => {
            if matches!(
                (
                    directory_with_module_path
                        .parent()
                        .as_ref()
                        .and_then(|path| path.name_and_extension()),
                    directory_with_module_path.name_and_extension(),
                ),
                (Some(("src", None)), Some(("bin", None)))
            ) {
                // files in /src/bin/ can import each other directly
                Some(directory_with_module_path)
            } else {
                directory_with_module_path.join(regular_rust_file_name)
            }
        }
    }?;

    module_chain_to_containing_module_file(module, db)
        .into_iter()
        .filter_map(|module| module.name(db))
        .try_fold(base_directory, |path, name| path.join(&name.to_string()))
}

fn module_chain_to_containing_module_file(
    current_module: Module,
    db: &RootDatabase,
) -> Vec<Module> {
    let mut path =
        iter::successors(Some(current_module), |current_module| current_module.parent(db))
            .take_while(|current_module| {
                matches!(current_module.definition_source(db).value, ModuleSource::Module(_))
            })
            .collect::<Vec<_>>();
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use crate::{test_utils::completion_list, CompletionKind};
    use expect_test::{expect, Expect};

    fn check(ra_fixture: &str, expect: Expect) {
        let actual = completion_list(ra_fixture, CompletionKind::Magic);
        expect.assert_eq(&actual);
    }

    #[test]
    fn lib_module_completion() {
        check(
            r#"
            //- /lib.rs
            mod $0
            //- /foo.rs
            fn foo() {}
            //- /foo/ignored_foo.rs
            fn ignored_foo() {}
            //- /bar/mod.rs
            fn bar() {}
            //- /bar/ignored_bar.rs
            fn ignored_bar() {}
        "#,
            expect![[r#"
                md foo;
                md bar;
            "#]],
        );
    }

    #[test]
    fn no_module_completion_with_module_body() {
        check(
            r#"
            //- /lib.rs
            mod $0 {

            }
            //- /foo.rs
            fn foo() {}
        "#,
            expect![[r#""#]],
        );
    }

    #[test]
    fn main_module_completion() {
        check(
            r#"
            //- /main.rs
            mod $0
            //- /foo.rs
            fn foo() {}
            //- /foo/ignored_foo.rs
            fn ignored_foo() {}
            //- /bar/mod.rs
            fn bar() {}
            //- /bar/ignored_bar.rs
            fn ignored_bar() {}
        "#,
            expect![[r#"
                md foo;
                md bar;
            "#]],
        );
    }

    #[test]
    fn main_test_module_completion() {
        check(
            r#"
            //- /main.rs
            mod tests {
                mod $0;
            }
            //- /tests/foo.rs
            fn foo() {}
        "#,
            expect![[r#"
                md foo
            "#]],
        );
    }

    #[test]
    fn directly_nested_module_completion() {
        check(
            r#"
            //- /lib.rs
            mod foo;
            //- /foo.rs
            mod $0;
            //- /foo/bar.rs
            fn bar() {}
            //- /foo/bar/ignored_bar.rs
            fn ignored_bar() {}
            //- /foo/baz/mod.rs
            fn baz() {}
            //- /foo/moar/ignored_moar.rs
            fn ignored_moar() {}
        "#,
            expect![[r#"
                md bar
                md baz
            "#]],
        );
    }

    #[test]
    fn nested_in_source_module_completion() {
        check(
            r#"
            //- /lib.rs
            mod foo;
            //- /foo.rs
            mod bar {
                mod $0
            }
            //- /foo/bar/baz.rs
            fn baz() {}
        "#,
            expect![[r#"
                md baz;
            "#]],
        );
    }

    // FIXME binary modules are not supported in tests properly
    // Binary modules are a bit special, they allow importing the modules from `/src/bin`
    // and that's why are good to test two things:
    // * no cycles are allowed in mod declarations
    // * no modules from the parent directory are proposed
    // Unfortunately, binary modules support is in cargo not rustc,
    // hence the test does not work now
    //
    // #[test]
    // fn regular_bin_module_completion() {
    //     check(
    //         r#"
    //         //- /src/bin.rs
    //         fn main() {}
    //         //- /src/bin/foo.rs
    //         mod $0
    //         //- /src/bin/bar.rs
    //         fn bar() {}
    //         //- /src/bin/bar/bar_ignored.rs
    //         fn bar_ignored() {}
    //     "#,
    //         expect![[r#"
    //             md bar;
    //         "#]],foo
    //     );
    // }

    #[test]
    fn already_declared_bin_module_completion_omitted() {
        check(
            r#"
            //- /src/bin.rs crate:main
            fn main() {}
            //- /src/bin/foo.rs
            mod $0
            //- /src/bin/bar.rs
            mod foo;
            fn bar() {}
            //- /src/bin/bar/bar_ignored.rs
            fn bar_ignored() {}
        "#,
            expect![[r#""#]],
        );
    }
}
