use scraper::{ElementRef, Html, Selector};
pub struct HtmlDocument {
    document: Html,
}

impl HtmlDocument {
    pub fn new(html: &str) -> Self {
        HtmlDocument {
            document: Html::parse_document(html),
        }
    }

    pub fn query_selector<'a, 'b>(&'a self, selector: &'b str) -> Option<ElementRef<'a>> {
        let s = Selector::parse(selector).ok()?;

        match self.document.select(&s).next() {
            Some(element) => Some(element),
            None => None,
        }
    }

    pub fn query_selector_all<'a>(&'a self, selector: &'a str) -> Option<Vec<ElementRef<'a>>> {
        let s = Selector::parse(selector).ok()?;

        let elements = self.document.select(&s).collect::<Vec<ElementRef>>();

        if elements.len() > 0 {
            Some(elements)
        } else {
            None
        }
    }

    pub fn get_element_by_id<'a>(&'a self, id: &'a str) -> Option<ElementRef<'a>> {
        self.query_selector(&format!("#{}", id))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_selector() {
        let html = r#"
            <html>
                <head>
                    <title>Test</title>
                </head>
                <body>
                    <div class="test" data-value="1">test1</div>
                    <div class="test" data-value="2">test2</div>
                </body>
            </html>
        "#;

        let document = HtmlDocument::new(html);

        assert_eq!(
            document
                .query_selector(".test[data-value='2']")
                .unwrap()
                .value()
                .attr("data-value")
                .unwrap(),
            "2"
        );
    }

    #[test]
    fn test_query_selector_all() {
        let html = r#"
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <div class="test" data-value="1">test1</div>
                <div class="test" data-value="2">test2</div>
                <div class="test" data-value="3">test3</div>
            </body>
        </html>
    "#;

        let document = HtmlDocument::new(html);

        let tests = document.query_selector_all(".test").unwrap();

        assert_eq!(tests.len(), 3);

        for (i, test) in tests.iter().enumerate() {
            assert_eq!(
                test.value().attr("data-value").unwrap(),
                format!("{}", i + 1)
            );
        }
    }

    #[test]
    fn test_get_element_by_id() {
        let html = r#"
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <div id="test1" data-value="1">test1</div>
                <div id="test2" data-value="2">test2</div>
                <div id="test3" data-value="3">test3</div>
            </body>
        </html>
    "#;

        let document = HtmlDocument::new(html);

        assert_eq!(
            document
                .get_element_by_id("test2")
                .unwrap()
                .value()
                .attr("data-value")
                .unwrap(),
            "2"
        );
    }
}
