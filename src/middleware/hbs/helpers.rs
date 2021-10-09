use handlebars::{Handlebars, Helper, Context, RenderContext, Output, RenderError};

pub fn url_encode_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // get parameter from helper or throw an error
    let s: String = h.param(0)
        .and_then(|param| Some(param.render()))
        .ok_or(RenderError::new("Input parameter 0 is required for url_encode helper."))?;
    
    let rendered = urlencoding::encode(s.as_str());

    out.write(rendered.as_ref())?;

    Ok(())
}

pub fn url_decode_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    // get parameter from helper or throw an error
    let s: String = h.param(0)
        .and_then(|param| Some(param.render()))
        .ok_or(RenderError::new("Input parameter 0 is required for url_encode helper."))?;
    
    let rendered = urlencoding::decode(s.as_str())
        .map_err(|err| RenderError::new(format!("Serialization Error: {:?}", err)))?;

    out.write(rendered.as_ref())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use handlebars::Handlebars;

    use super::*;

    #[test]
    fn url_encode_test() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("url_encode", Box::new(url_encode_helper));

        let rendered = hbs.render_template(r##"{{ url_encode "one two three" }}"##, &()).unwrap();
        assert_eq!(r##"one%20two%20three"##, rendered);
    }

    #[test]
    fn url_decode_test() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("url_decode", Box::new(url_decode_helper));

        let rendered = hbs.render_template(r##"{{ url_decode "one%20two%20three" }}"##, &()).unwrap();
        assert_eq!(r##"one two three"##, rendered);
    }
}