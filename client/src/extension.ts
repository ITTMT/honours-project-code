import * as vscode from 'vscode';
import * as path from 'path';
import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	RequestType,
	DocumentSelector
} from 'vscode-languageclient/node';

let client: LanguageClient;
let windows: vscode.TextEditor[] = new Array();

export function activate(context: vscode.ExtensionContext) {
	const traceOutputChannel = vscode.window.createOutputChannel("BHC LSP Trace");
	const command = process.env.SERVER_PATH || "bhc-language-server";

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
		documentSelector: [{scheme: "file", language: "html"}],
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher("**/.clientrc"),
		},
		traceOutputChannel,
	};

	client = new LanguageClient("bhc-language-server", "bhc language server", serverOptions, clientOptions);
	client.start();
	
	// Custom command to open a text document, need this because I want to be able to open a window side by side to a document rather than override a currently open document.
	client.onRequest("bhc/ShowDocumentRequest", async (handler: BhcShowDocumentParams) => {
		await vscode.workspace.openTextDocument(handler.uri).then(async document => {
			await vscode.window.showTextDocument(document, {
				viewColumn: vscode.ViewColumn.Two,
				preserveFocus: true
			})
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
	uri: string
}
``

function xyz(event) {

	let x = vscode.window.tabGroups.all.flatMap(({ tabs }) => tabs.map(tab => tab.group))

	console.log(x);

	console.log(event);
}

// function abc(event) {

// 	console.log(event);
// }

// vscode.window.onDidChangeActiveTextEditor(abc);

vscode.workspace.onDidOpenTextDocument(xyz);
// vscode.workspace.onDidCloseTextDocument(xyz);


// rule 1, if we open a file, we open its corresponding css file in column 2
// rule 2, if we open another file, we open its corresponding css file in column 2 as well
// rule 3, if we change focus of the html file to another one, we change focus of the css file to the corresponding file.
// rule 4, rule 3 vice versa.
// rule 5, if we try to open a file inside column 2, we open it inside column 1. Make sure that none of the active files have been closed.