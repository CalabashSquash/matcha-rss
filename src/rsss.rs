pub mod rsss {
    use std::io::{BufRead, Write};
    use std::borrow::Cow;
    use rss::{Channel, Error, Item, Image, TextInput};
    use quick_xml::Reader;
    use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event, attributes::Attributes};
    use quick_xml::Error as XmlError;
    use std::collections::BTreeMap;

    pub(crate) fn decode<'s, B: BufRead>(
        bytes: &'s [u8],
        reader: &Reader<B>,
    ) -> Result<Cow<'s, str>, Error> {
        let text = reader.decoder().decode(bytes)?;
        Ok(text)
    }

    fn read_namespace_declarations<'m, R>(
        reader: &mut Reader<R>,
        mut atts: Attributes,
        base: &'m BTreeMap<String, String>,
    ) -> Result<Cow<'m, BTreeMap<String, String>>, Error>
    where
        R: BufRead,
    {
        let mut namespaces = Cow::Borrowed(base);
        for attr in atts.with_checks(false).flatten() {
            let key = decode(attr.key.as_ref(), reader)?;
            if let Some(ns) = key.strip_prefix("xmlns:") {
                namespaces
                    .to_mut()
                    .insert(ns.to_string(), attr_value(&attr, reader)?.to_string());
            }
        }
        Ok(namespaces)
    }

    pub fn read_from<R: BufRead>(reader: R) -> Result<Channel, Error> {
        let mut reader = Reader::from_reader(reader);
        reader.config_mut().expand_empty_elements = true;
        let namespaces;
        let mut buf = Vec::new();

        let mut channel: Option<Channel> = None;

        // for parsing RSS 0.9, 1.0 feeds
        let mut items: Option<Vec<Item>> = None;
        let mut image: Option<Image> = None;
        let mut text_input: Option<TextInput> = None;

        // find opening element
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(element) => match decode(element.name().as_ref(), &reader)?.as_ref() {
                    "rss" | "rdf:RDF" => {
                        namespaces = read_namespace_declarations(
                            &mut reader,
                            element.attributes(),
                            &BTreeMap::new(),
                        )?
                        .into_owned();
                        break;
                    }
                    _ => {
                        return Err(Error::InvalidStartTag);
                    }
                },
                Event::Eof => return Err(Error::Eof),
                _ => continue,
            }
        }

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(element) => match decode(element.name().as_ref(), &reader)?.as_ref() {
                    "channel" => {
                        let inner =
                            Channel::from_xml(&namespaces, &mut reader, element.attributes())?;
                        channel = Some(inner);
                    }
                    "item" => {
                        let item = Item::from_xml(&namespaces, &mut reader, element.attributes())?;
                        if items.is_none() {
                            items = Some(Vec::new());
                        }
                        items.as_mut().unwrap().push(item);
                    }
                    "image" => {
                        let inner = Image::from_xml(&mut reader, element.attributes())?;
                        image = Some(inner);
                    }
                    "textinput" => {
                        let inner = TextInput::from_xml(&mut reader, element.attributes())?;
                        text_input = Some(inner);
                    }
                    _ => skip(element.name(), &mut reader)?,
                },
                Event::End(_) | Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        if let Some(mut channel) = channel {
            if let Some(mut items) = items {
                channel.items.append(&mut items);
            }

            if image.is_some() {
                channel.image = image;
            }

            if text_input.is_some() {
                channel.text_input = text_input;
            }

            channel.namespaces = namespaces;

            Ok(channel)
        } else {
            Err(Error::Eof)
        }
    }
}