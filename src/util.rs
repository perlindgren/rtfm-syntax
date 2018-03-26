use syn::{Ident, Path, PathArguments, PathSegment, punctuated::Punctuated};

/// Creates a path with contents `#ident`
pub fn mk_path(ident: &str) -> Path {
    let ident = Ident::from(ident);
    let path_segment = PathSegment {
        ident,
        arguments: PathArguments::None,
    };
    let mut segments: Punctuated<Ident, Token![::]> = Punctuated::new();
    segments.push(path_segment);

    Path {
        leading_colon: None,
        segments,
    }
}
