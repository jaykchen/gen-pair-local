# Text Segmentation and LLM Training Sample Generation

This project achieves the goal in 2 stages and multiple steps within each stage.

## Stage One: Text Segmentation with Pandoc

In the initial stage, we utilize the well-known document conversion tool [Pandoc](http://pandoc.org) to convert the source document into segments of plain text.

Pandoc's primary function is to translate documents from one format to another. For this project, our target format is segmented plain text. We employ a custom Python filter, adhering to Pandoc's filter conventions, to extract plain text from the internal text blocks that Pandoc identifies within various typesetting elements, such as paragraphs and tables. This filter generates an "segmented_text.json" file containing the list of plain text segments we require.

Step 1: Install Pandoc as indicated in the [Pandoc User's Guide](http://pandoc.org/installing.html)

Step 2: Install the Python dependencies for the custom Pandoc filter

Step 3: Use Pandoc in command line mode to convert the source document into plain text and store the result in the "book.txt" file. An example command line is shown below.
```sh
pandoc -s book.txt --filter filter.py -o output.txt

Producing the output.txt with the above command line is just a hack to make Pandoc work for us


## Stage Two: QA Pair Generation with OpenAI

Step 1: run the Rust project

Please mannually specify the .json file that contains the list of segmented text generated from the previous stage in the main.rs file

