import { LoadingOutlined, PlayCircleOutlined } from "@ant-design/icons";
import { autocompletion, snippetCompletion, type CompletionContext } from "@codemirror/autocomplete";
import { StreamLanguage, indentUnit } from "@codemirror/language";
import { linter, lintGutter, type Diagnostic } from "@codemirror/lint";
import type { Text } from "@codemirror/state";
import { EditorView, placeholder as editorPlaceholder } from "@codemirror/view";
import CodeMirror from "@uiw/react-codemirror";
import { Button, Input, Modal, Tooltip } from "antd";
import {
  useCallback,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import { useTranslation } from "react-i18next";
import { useDispatch } from "react-redux";
import { useMessageContext } from "../../hooks";
import { setIsLoading } from "../../store/other";
import { requestPost } from "../../utils";

type ScriptDiagnosticSpan = {
  startLine: number;
  startCol: number;
  endLine: number;
  endCol: number;
};

type ScriptDiagnostic = {
  severity: "error";
  code: string;
  message: string;
  span: ScriptDiagnosticSpan;
  related?: {
    message: string;
    span: ScriptDiagnosticSpan;
  }[];
};

type ScriptValidateResult = {
  valid: boolean;
  diagnostics: ScriptDiagnostic[];
};

type ScriptEditorProps = {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  minRows?: number;
  maxRows?: number;
  showRun?: boolean;
};

const KEYWORDS = ["let", "if", "else", "while", "true", "false"];
const BUILTIN_VARS = [
  "ORIGINAL_W",
  "ORIGINAL_H",
  "CURSOR_X",
  "CURSOR_Y",
  "RawInputFlag",
  "FpsModeFlag",
];

const BUILTIN_FUNCTIONS = [
  { label: "print", detail: "(...values)", apply: "print(${})" },
  { label: "wait", detail: "(ms)", apply: "wait(${ms})" },
  { label: "tap", detail: "(pointerId, x, y, action?)", apply: "tap(${pointerId}, ${x}, ${y})" },
  { label: "swipe", detail: "(pointerId, duration, x1, y1, ...)", apply: "swipe(${pointerId}, ${duration}, ${x1}, ${y1}, ${x2}, ${y2})" },
  { label: "send_key", detail: "(key, action?, metastate?)", apply: "send_key(${key})" },
  { label: "paste_text", detail: "(text)", apply: "paste_text(${text})" },
  { label: "state_set", detail: "(key, value)", apply: "state_set(${key}, ${value})" },
  { label: "state_get", detail: "(key, defaultValue)", apply: "state_get(${key}, ${defaultValue})" },
  { label: "state_has", detail: "(key)", apply: "state_has(${key})" },
  { label: "state_delete", detail: "(key)", apply: "state_delete(${key})" },
  { label: "state_clear", detail: "()", apply: "state_clear()" },
  { label: "enter_fps", detail: "(id)", apply: "enter_fps(${id})" },
  { label: "exit_fps", detail: "()", apply: "exit_fps()" },
  { label: "enter_raw_input", detail: "()", apply: "enter_raw_input()" },
  { label: "exit_raw_input", detail: "()", apply: "exit_raw_input()" },
  { label: "cancel_cast", detail: "(id)", apply: "cancel_cast(${id})" },
  { label: "release_cast", detail: "()", apply: "release_cast()" },
];

const scriptLanguage = StreamLanguage.define({
  token(stream) {
    if (stream.eatSpace()) return null;
    if (stream.match("//")) {
      stream.skipToEnd();
      return "comment";
    }
    if (stream.match(/"(?:\\.|[^"\\])*"?/)) return "string";
    if (stream.match(/\d+/)) return "number";
    if (stream.match(/\b(?:let|if|else|while|true|false)\b/)) return "keyword";
    if (stream.match(/[A-Za-z_][A-Za-z0-9_]*(?=\s*\()/)) return "function(variableName)";
    if (stream.match(/[A-Za-z_][A-Za-z0-9_]*/)) return "variableName";
    if (stream.match(/==|!=|<=|>=|\|\||&&|[+\-*/%<>=!]/)) return "operator";
    stream.next();
    return null;
  },
  languageData: {
    commentTokens: { line: "//" },
  },
});

function scriptCompletionSource(context: CompletionContext) {
  const word = context.matchBefore(/[A-Za-z_][A-Za-z0-9_]*/);
  if (!word && !context.explicit) return null;

  const localVars = Array.from(
    context.state.doc.toString().matchAll(/\blet\s+([A-Za-z_][A-Za-z0-9_]*)/g),
    (match) => match[1],
  );

  const options = [
    ...KEYWORDS.map((label) => ({ label, type: "keyword" })),
    ...localVars.map((label) => ({ label, type: "variable", detail: "local" })),
    ...BUILTIN_VARS.map((label) => ({ label, type: "variable" })),
    ...BUILTIN_FUNCTIONS.map((item) =>
      snippetCompletion(item.apply, {
        label: item.label,
        detail: item.detail,
        type: "function",
      }),
    ),
    snippetCompletion("if ${condition} {\n\t${}\n}", {
      label: "if block",
      type: "keyword",
      detail: "if (...)",
    }),
    snippetCompletion("while ${condition} {\n\t${}\n}", {
      label: "while block",
      type: "keyword",
      detail: "while (...)",
    }),
  ];

  return {
    from: word?.from ?? context.pos,
    options,
  };
}

function toPosition(doc: Text, line: number, col: number) {
  const lineInfo = doc.line(Math.min(Math.max(line, 1), doc.lines));
  return Math.min(lineInfo.to, lineInfo.from + Math.max(col - 1, 0));
}

function toDiagnostic(doc: Text, diagnostic: ScriptDiagnostic): Diagnostic {
  const from = toPosition(doc, diagnostic.span.startLine, diagnostic.span.startCol);
  let to = toPosition(doc, diagnostic.span.endLine, diagnostic.span.endCol);
  if (to <= from) {
    to = Math.min(doc.length, from + 1);
  }

  const related = diagnostic.related ?? [];
  const message =
    related.length === 0
      ? diagnostic.message
      : [
          diagnostic.message,
          ...related.map(
            (item) =>
              `${item.message} (line ${item.span.startLine}, column ${item.span.startCol})`,
          ),
        ].join("\n");

  return {
    from,
    to,
    severity: diagnostic.severity,
    message,
  };
}

async function validateScript(view: EditorView): Promise<Diagnostic[]> {
  const script = view.state.doc.toString();
  if (script.trim() === "") return [];

  const res = await requestPost<ScriptValidateResult>("/api/script/validate", {
    script,
  });

  return res.data.diagnostics.map((diagnostic) =>
    toDiagnostic(view.state.doc, diagnostic),
  );
}

export function ScriptEditor({
  value,
  onChange,
  placeholder,
  className,
  minRows = 1,
  maxRows = 10,
  showRun = true,
}: ScriptEditorProps) {
  const { t } = useTranslation();
  const dispatch = useDispatch();
  const messageApi = useMessageContext();
  const [runningError, setRunningError] = useState("");
  const [running, setRunning] = useState(false);
  const [hasVerticalScrollbar, setHasVerticalScrollbar] = useState(false);
  const editorRef = useRef<HTMLDivElement>(null);

  const updateScrollbarState = useCallback(() => {
    const scroller =
      editorRef.current?.querySelector<HTMLElement>(".cm-scroller");
    if (!scroller) return;
    setHasVerticalScrollbar(scroller.scrollHeight > scroller.clientHeight + 1);
  }, []);

  const extensions = useMemo(() => {
    const lineHeight = 22;
    return [
      scriptLanguage,
      indentUnit.of("  "),
      EditorView.lineWrapping,
      EditorView.theme({
        "&": {
          border: "1px solid var(--ant-color-border)",
          borderRadius: "var(--ant-border-radius)",
          backgroundColor: "var(--ant-color-bg-container)",
          fontSize: "var(--ant-font-size)",
        },
        "&.cm-focused": {
          outline: "none",
          borderColor: "var(--ant-color-primary)",
        },
        ".cm-scroller": {
          fontFamily: "var(--ant-font-family-code)",
          minHeight: `${minRows * lineHeight + 12}px`,
          maxHeight: `${maxRows * lineHeight + 12}px`,
          overflow: "auto",
        },
        ".cm-content": {
          padding: "6px var(--script-editor-content-padding) 6px 0",
        },
        ".cm-gutters": {
          backgroundColor: "var(--ant-color-bg-container)",
          borderRightColor: "var(--ant-color-border-secondary)",
        },
        ".cm-tooltip": {
          backgroundColor: "var(--ant-color-bg-elevated)",
          borderColor: "var(--ant-color-border)",
          color: "var(--ant-color-text)",
        },
        ".cm-diagnosticText": {
          whiteSpace: "pre-wrap",
        },
        ".cm-placeholder": {
          color: "var(--ant-color-text-placeholder)",
        },
      }),
      editorPlaceholder(placeholder ?? ""),
      autocompletion({
        override: [scriptCompletionSource],
        activateOnTyping: true,
      }),
      linter(validateScript, {
        delay: 400,
      }),
      lintGutter(),
    ];
  }, [maxRows, minRows, placeholder]);

  useLayoutEffect(() => {
    updateScrollbarState();

    const scroller =
      editorRef.current?.querySelector<HTMLElement>(".cm-scroller");
    const content = editorRef.current?.querySelector<HTMLElement>(".cm-content");
    if (!scroller) return;

    const resizeObserver = new ResizeObserver(updateScrollbarState);
    resizeObserver.observe(scroller);
    if (content) resizeObserver.observe(content);

    const frame = requestAnimationFrame(updateScrollbarState);
    return () => {
      cancelAnimationFrame(frame);
      resizeObserver.disconnect();
    };
  }, [maxRows, minRows, updateScrollbarState, value]);

  const editorStyle = {
    "--script-editor-content-padding": hasVerticalScrollbar ? "50px" : "34px",
  } as CSSProperties;

  async function runScript() {
    setRunning(true);
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/script/run", { script: value });
      messageApi?.success(res.message);
    } catch (error: any) {
      setRunningError(error);
    } finally {
      setRunning(false);
      dispatch(setIsLoading(false));
    }
  }

  return (
    <div className={className}>
      <Modal
        title={t("mappings.common.scriptEditor.runResult")}
        className="min-w-50vw"
        open={runningError !== ""}
        onCancel={() => setRunningError("")}
        footer={null}
      >
        <Input.TextArea
          className="font-mono"
          value={runningError}
          readOnly
          autoSize
        />
      </Modal>
      <div ref={editorRef} className="group relative" style={editorStyle}>
        <CodeMirror
          value={value}
          height="auto"
          theme="dark"
          basicSetup={{
            autocompletion: false,
            foldGutter: false,
            highlightActiveLine: false,
            highlightActiveLineGutter: false,
          }}
          extensions={extensions}
          onChange={onChange}
        />
        {showRun && (
          <Tooltip title={t("mappings.common.scriptEditor.run")}>
            <Button
              className="absolute top-1.5 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100"
              style={{ right: hasVerticalScrollbar ? 16 : 6 }}
              size="small"
              type="text"
              icon={running ? <LoadingOutlined /> : <PlayCircleOutlined />}
              disabled={running || value.trim() === ""}
              onClick={runScript}
            />
          </Tooltip>
        )}
      </div>
    </div>
  );
}
