
#[derive(Debug)]
pub enum ProductError {
    NoSubProduct,
    BadSubProduct,
    BadProduct
}

impl From<strum::ParseError> for ProductError {
    fn from(_: strum::ParseError) -> Self {
        Self::BadSubProduct
    }
}

#[derive(Debug)]
pub enum ParseError {
    /// Error parsing an enum variant.
    Enum,
    /// Error parsing a commodity product.
    Product(ProductError)
}

impl From<strum::ParseError> for ParseError {
    fn from(_: strum::ParseError) -> Self {
        Self::Enum
    }
}

impl From<ProductError> for ParseError {
    fn from(e: ProductError) -> Self {
        Self::Product(e)
    }
}