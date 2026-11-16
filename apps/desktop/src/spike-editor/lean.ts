// Markdown language without the HTML/CSS/JS sub-languages of @codemirror/lang-markdown:
// the @lezer/markdown parser (CommonMark + GFM) wrapped as a CodeMirror Language.
import { defineLanguageFacet, foldNodeProp, indentNodeProp, Language, languageDataProp } from "@codemirror/language";
import { GFM, parser as baseParser } from "@lezer/markdown";

const data = defineLanguageFacet({ commentTokens: { block: { open: "<!--", close: "-->" } } });

const parser = baseParser.configure([
  GFM,
  {
    props: [
      foldNodeProp.add((type) => (type.is("Block") && !type.is("Document") ? (tree, state) => ({ from: state.doc.lineAt(tree.from).to, to: tree.to }) : undefined)),
      indentNodeProp.add({ Document: () => null }),
      languageDataProp.add({ Document: data }),
    ],
  },
]);

export const leanMarkdown = new Language(data, parser, [], "markdown");
