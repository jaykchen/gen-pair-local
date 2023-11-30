#!/usr/bin/env python3
import panflute as pf
import json

def action(elem, doc):
    # Check if the element is a header, which signifies the start of a new segment
    if isinstance(elem, pf.Header):
        if doc.current_segment:
            doc.collected_texts.append(doc.current_segment)
            doc.current_segment = []
    
    # Check if the element is a block-level element that should be included in the segments
    if isinstance(elem, (pf.Header, pf.Para, pf.BlockQuote, pf.Div, pf.Plain,
                         pf.DefinitionList, pf.BulletList, pf.OrderedList, 
                         pf.LineBlock, pf.CodeBlock, pf.RawBlock)):
        # Append the content to the current segment
        doc.current_segment.append(pf.stringify(elem))

def finalize(doc):
    # Add the last segment if it exists
    if doc.current_segment:
        doc.collected_texts.append(doc.current_segment)

    # Export the collected text content to a JSON file
    with open('segmented_text.json', 'w') as json_file:
        json.dump(doc.collected_texts, json_file, ensure_ascii=False, indent=4)

def prepare(doc):
    # Initialize a list to collect text elements
    doc.collected_texts = []
    # Initialize the current segment
    doc.current_segment = []

def main(doc=None):
    return pf.run_filter(action, prepare=prepare, finalize=finalize, doc=doc)

if __name__ == '__main__':
    main()
