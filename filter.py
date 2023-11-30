import panflute as pf
import json

def action(elem, doc):
    # Collect only the plain text content of paragraphs, code blocks, or block quotes
    if isinstance(elem, (pf.Para, pf.CodeBlock, pf.BlockQuote, pf.BulletList, pf.OrderedList, pf.DefinitionList,   pf.Table)):
        text = pf.stringify(elem)
        doc.collected_texts.append(text)

def finalize(doc):
    # Export the collected text content to a JSON file
    with open('segmented_text.json', 'w') as json_file:
        json.dump(doc.collected_texts, json_file)

def prepare(doc):
    # Initialize a list to collect text elements
    doc.collected_texts = []

def main(doc=None):
    return pf.run_filter(action, prepare=prepare, finalize=finalize, doc=doc)

if __name__ == '__main__':
    main()
