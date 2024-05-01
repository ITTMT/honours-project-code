import { start } from 'repl';
import * as vscode from 'vscode';
import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;
let windows: vscode.TextEditor[] = new Array();

export function activate() {
	const command = process.env.SERVER_PATH || "bhc-language-server";

	vscode.commands.registerCommand('bhc.activate', () => {})

	const run: Executable = {
		command,
		options: {
			env: {
				...process.env,
				RUST_LOG:"debug",
			},
		},
	};

	const serverOptions: ServerOptions = {
		run,
		debug: run,
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [
			{scheme: "file", language: "html"},
			{scheme: "file", language: "css"}
		],
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher("**/.clientrc"),
		},
	};

	client = new LanguageClient("bhc-language-server", "bhc language server", serverOptions, clientOptions);

	client.start();

	const colorChoices: vscode.TextEditorDecorationType[] = [];

	// currently only 10 max files, maybe do some pseudo random color generation. I want the same colours to be present every time.
	const colors = [
		'#FF000050', 
		'#00FF0050', 
		'#0000FF50', 
		'#FFFF0050', 
		'#FF00FF50', 
		'#00FFFF50', 
		'#FFFFFF50', 
		'#50008550', 
		'#FF800050',
		'#69003650'
	]

	for (let color of colors) {
		colorChoices.push(
			vscode.window.createTextEditorDecorationType({
				rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
				backgroundColor: color
			})
		);
	}

	// Custom command to open a text document, need this because I want to be able to open a window side by side to a document rather than override a currently open document.
	client.onRequest("bhc/ShowDocumentRequest", async (handler: BhcShowDocumentParams) => {
		await vscode.workspace.openTextDocument(handler.uri).then(async document => {
			let editor = await vscode.window.showTextDocument(document, {
				viewColumn: vscode.ViewColumn.Two,
				preserveFocus: true
			});

			const uniqueIds: number[] = [... new Set(handler.file.included_files.map(x => x.id))];

			const ownerKeyValues: Record<number, number> = uniqueIds.reduce((acc, ownerId, index) => {
				acc[ownerId] = index;
				return acc;
			}, {});

			let decorations: vscode.DecorationOptions[][] = [];

			for (const line of handler.file.lines) {
				if (line.owner != null) {
					const fill_position = editor.document.lineAt(line.line_number);

					if (!decorations[ownerKeyValues[line.owner]]) {
						decorations[ownerKeyValues[line.owner]] = [];
					}

					const decoration = { range: new vscode.Range(
						fill_position.lineNumber, 
						fill_position.firstNonWhitespaceCharacterIndex, 
						fill_position.lineNumber, 
						fill_position.range.end.character
						)
					}

					decorations[ownerKeyValues[line.owner]].push(decoration);
				}
			}

			for (const key in ownerKeyValues) {
				editor.setDecorations(colorChoices[ownerKeyValues[key]], decorations[ownerKeyValues[key]]);
			}
		});
	});

}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}


type BhcShowDocumentParams = {
	uri: vscode.Uri,
	file: FormattedCssFile
}

type FormattedCssFile = {
	included_files: FileMetaData[],
	styles: CssStyleExtended[],
	lines: LineInformation[]
}

type FileMetaData = {
	id: number,
	file_name: string,
	absolute_path: string
}

type CssStyleExtended = {
	owner: number | null,
	tag: string,
	attributes: CssAttributeExtended[]
}

type LineInformation = {
	line_number: number,
	owner: number | null
}

type CssAttributeExtended = {
	owner: number,
	name: string,
	value: string,
	source: number | null
	is_overwritten: boolean | null
}

// rule 1, if we open a file, we open its corresponding css file in column 2
// rule 2, if we open another file, we open its corresponding css file in column 2 as well
// rule 3, if we change focus of the html file to another one, we change focus of the css file to the corresponding file.
// rule 4, rule 3 vice versa.
// rule 5, if we try to open a file inside column 2, we open it inside column 1. Make sure that none of the active files have been closed.

// there's a weird event happening that an automatically opened file isn't signalling when it is closed 100% of the time.
// need to link the html file to the virtual css file so it opens and closes with eachother