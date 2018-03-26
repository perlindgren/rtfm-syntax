use syn::{Ident, Path, PathArguments, PathSegment, punctuated::Punctuated,
          token::Colon2};

/// Creates a path with contents `#ident`
pub fn mk_path(ident: &str) -> Path {
    let ident = Ident::from(ident);
    let path_segment = PathSegment {
        ident,
        arguments: PathArguments::None,
    };

    let mut segments: Punctuated<PathSegment, Colon2> = Punctuated::new();
    segments.push(path_segment);

    Path {
        leading_colon: None,
        segments,
    }
}
