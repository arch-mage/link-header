use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Write;
use std::str::FromStr;

use headers::Header;
use http::header::HeaderValue;
use http::header::LINK;
use http::Uri;

#[derive(Debug)]
pub struct Link(Box<[LinkItem]>);

impl<const N: usize> From<[LinkItem; N]> for Link {
    fn from(value: [LinkItem; N]) -> Self {
        Link(Box::from(value))
    }
}

impl FromIterator<LinkItem> for Link {
    fn from_iter<T: IntoIterator<Item = LinkItem>>(iter: T) -> Self {
        Link(iter.into_iter().collect())
    }
}

impl FromStr for Link {
    type Err = InvalidLink;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .map(LinkItem::from_str)
            .collect::<Result<Box<[LinkItem]>, Self::Err>>()
            .map(Link)
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut links = self.0.iter();

        if let Some(link) = links.next() {
            link.fmt(f)?;
            for link in links {
                f.write_char(',')?;
                link.fmt(f)?;
            }
        };
        Ok(())
    }
}

impl Header for Link {
    fn name() -> &'static headers::HeaderName {
        &LINK
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .ok_or_else(headers::Error::invalid)?
            .to_str()
            .map_err(|_| InvalidLink)
            .and_then(Self::from_str)
            .map_err(Into::into)
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let s = self.to_string();
        let value = HeaderValue::from_str(s.as_str()).unwrap();

        values.extend(std::iter::once(value));
    }
}

#[derive(Debug)]
pub struct LinkItem {
    uri: Uri,
    params: HashMap<String, String>,
}

impl LinkItem {
    pub fn new(uri: Uri) -> LinkItem {
        LinkItem {
            uri,
            params: HashMap::default(),
        }
    }

    pub fn with_param<I, K, V>(uri: Uri, params: I) -> LinkItem
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        LinkItem {
            uri,
            params: params
                .into_iter()
                .map(|(key, val)| (key.into(), val.into()))
                .collect(),
        }
    }

    #[inline]
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    #[inline]
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|param| param.as_str())
    }
}

impl FromStr for LinkItem {
    type Err = InvalidLink;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (link, parameters) = s
            .split_once(';')
            .map(|(a, b)| (a, Some(b)))
            .unwrap_or((s, None));
        let link = link.strip_prefix('<').ok_or(InvalidLink)?;
        let link = link.strip_suffix('>').ok_or(InvalidLink)?;
        let uri: Uri = link.parse().map_err(|_| InvalidLink)?;
        let mut params = HashMap::new();

        if let Some(parameters) = parameters {
            for param in parameters.split(';') {
                let (name, data) = param.trim().split_once('=').ok_or(InvalidLink)?;
                params.insert(name.to_string(), data.to_string());
            }
        };

        Ok(LinkItem { uri, params })
    }
}

impl Display for LinkItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_char('<')?;
        self.uri.fmt(f)?;
        f.write_char('>')?;
        for (name, data) in self.params.iter() {
            f.write_str("; ")?;
            f.write_str(name.as_str())?;
            f.write_char('=')?;
            f.write_str(data.as_str())?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidLink;

impl std::error::Error for InvalidLink {}

impl std::fmt::Display for InvalidLink {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("invalid link")
    }
}

impl From<InvalidLink> for headers::Error {
    #[inline]
    fn from(_: InvalidLink) -> Self {
        headers::Error::invalid()
    }
}
