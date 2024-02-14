#[macro_use]
extern crate tantivy;
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::error::TantivyError;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::IndexBuilder;
use tantivy::ReloadPolicy;

#[derive(Debug)]
enum XmlParsingError {
    Error(String),
}

fn node_to_field(
    field_type: &FieldType,
    node: roxmltree::Node,
) -> std::result::Result<Value, XmlParsingError> {
    match node.node_type() {
        roxmltree::NodeType::Root => match &field_type {
            _ => Err(XmlParsingError::Error(
                "Failed to convert root to value".to_string(),
            )),
        },
        roxmltree::NodeType::Element => match field_type {
            _ => Err(XmlParsingError::Error(
                "Failed to convert element to value".to_string(),
            )),
        },
        roxmltree::NodeType::PI => match field_type {
            _ => Err(XmlParsingError::Error(
                "Failed to convert processing instruction to value".to_string(),
            )),
        },
        roxmltree::NodeType::Comment => match field_type {
            _ => Err(XmlParsingError::Error(
                "Failed to convert comment to value".to_string(),
            )),
        },
        roxmltree::NodeType::Text => match field_type {
            FieldType::Str(_) => Ok(Value::Str(node.text().unwrap().to_string())),
            _ => Err(XmlParsingError::Error(
                "Failed to convert text to value".to_string(),
            )),
        },
    }
}

fn xml_to_tantivy(schema: Schema, xml: roxmltree::Document) -> Result<Document, XmlParsingError> {
    let mut doc = Document::default();
    if let Some(frontmatter) = xml
        .root_element()
        .children()
        .find(|n| n.tag_name() == "frontmatter".into())
    {
        let nodes = frontmatter.children();
        for node in nodes {
            let field_name = node.tag_name().name();
            if let Ok(field) = schema.get_field(&field_name) {
                let field_entry = schema.get_field_entry(field);
                let field_type: &FieldType = field_entry.field_type();
                println!("{:?}", field);
                println!("{:?}\n", field_entry);
                // println!("{:?}\n", field_type);
                let value = node_to_field(field_type, node)?;
                doc.add_field_value(field, value);
                // println!("got field!")
            } else {
                // println!("didn't get field!")
            }
        }
        // items.for_each(|i| doc.add_text());
        // println!("{:?}", items);
    }
    // .find(|n| n.tag_name() == "frontmatter".into())

    Ok(doc)
}

fn main() -> tantivy::Result<()> {
    let text = std::fs::read_to_string("./output/frct-0001.xml").unwrap();

    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    schema_builder.add_text_field("authors", TEXT);
    schema_builder.add_text_field("taxon", TEXT);
    schema_builder.add_u64_field("year", INDEXED);
    schema_builder.add_u64_field("month", INDEXED);
    schema_builder.add_u64_field("day", INDEXED);

    let schema = schema_builder.build();
    let doc = match roxmltree::Document::parse(&text) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    // println!("{:?}", xml_to_tantivy(schema, doc));
    let index_path = Path::new("./index");

    let index = IndexBuilder::new()
        .schema(schema)
        .open_or_create(index_path, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?;

    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();
    // let title = schema.get_field("taxon").unwrap();
    // let title = schema.get_field("author").unwrap();

    let mut old_man_doc = Document::default();
    old_man_doc.add_text(title, "The Old Man and the Sea");
    old_man_doc.add_text(
        body,
        "He was an old man who fished alone in a skiff in the Gulf Stream and \
        he had gone eighty-four days now without taking a fish.",
    );

    index_writer.add_document(old_man_doc);

    index_writer.add_document(doc!(
    title => "Of Mice and Men",
    body => "A few miles south of Soledad, the Salinas River drops in close to the hillside \
            bank and runs deep and green. The water is warm too, for it has slipped twinkling \
            over the yellow sands in the sunlight before reaching the narrow pool. On one \
            side of the river the golden foothill slopes curve up to the strong and rocky \
            Gabilan Mountains, but on the valley side the water is lined with trees—willows \
            fresh and green with every spring, carrying in their lower leaf junctures the \
            debris of the winter’s flooding; and sycamores with mottled, white, recumbent \
            limbs and branches that arch over the pool"
    ));

    index_writer.add_document(doc!(
    title => "Of Mice and Men",
    body => "A few miles south of Soledad, the Salinas River drops in close to the hillside \
            bank and runs deep and green. The water is warm too, for it has slipped twinkling \
            over the yellow sands in the sunlight before reaching the narrow pool. On one \
            side of the river the golden foothill slopes curve up to the strong and rocky \
            Gabilan Mountains, but on the valley side the water is lined with trees—willows \
            fresh and green with every spring, carrying in their lower leaf junctures the \
            debris of the winter’s flooding; and sycamores with mottled, white, recumbent \
            limbs and branches that arch over the pool"
    ));

    index_writer.add_document(doc!(
    title => "Frankenstein",
    title => "The Modern Prometheus",
    body => "You will rejoice to hear that no disaster has accompanied the commencement of an \
             enterprise which you have regarded with such evil forebodings.  I arrived here \
             yesterday, and my first task is to assure my dear sister of my welfare and \
             increasing confidence in the success of my undertaking."
    ));
    index_writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![title, body]);

    let query = query_parser.parse_query("sea whale")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
