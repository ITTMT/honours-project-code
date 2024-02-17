import * as vscode from 'vscode';
import * as path from 'path';
import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
	const disposable = vscode.commands.registerCommand("helloworld.helloWorld", async uri => {
		// The code you place here will be executed every time your command is executed
		// Display a message box to the user
		const url = vscode.Uri.parse('/home/victor/Documents/test-dir/nrs/another.nrs');
		const document = await vscode.workspace.openTextDocument(uri);
		await vscode.window.showTextDocument(document);
		
		// console.log(uri)
		vscode.window.activeTextEditor.document;
		const editor = vscode.window.activeTextEditor;
		const range = new vscode.Range(1, 1, 1, 1);
		editor.selection = new vscode.Selection(range.start, range.end);
	});

	const traceOutputChannel = vscode.window.createOutputChannel("LSP Trace");
	const command = process.env.SERVER_PATH || "server";

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
		documentSelector: [{ scheme: "file", language: "html"}], 
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher("**/.clientrc"),
		},
		traceOutputChannel,
	};

	client = new LanguageClient("bhc-language-server", "bhc language server", serverOptions, clientOptions);
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}

export function activateInlayHints(ctx: vscode.ExtensionContext) {
	const maybeUpdater = {
		hintsProvider: null as Disposable | null,
		updateHintsEventEmitter: new vscode.EventEmitter<void>(),
  
		async onConfigChange() {
			this.dispose();
  
			const event = this.updateHintsEventEmitter.event;
		},
  

		onDidChangeTextDocument({ contentChanges, document }: vscode.TextDocumentChangeEvent) {
		// debugger
		// this.updateHintsEventEmitter.fire();
		},
  
		dispose() {
			this.hintsProvider?.dispose();
			this.hintsProvider = null;
			this.updateHintsEventEmitter.dispose();
		},
	};
  
	vscode.workspace.onDidChangeConfiguration(maybeUpdater.onConfigChange, maybeUpdater, ctx.subscriptions);
	vscode.workspace.onDidChangeTextDocument(maybeUpdater.onDidChangeTextDocument, maybeUpdater, ctx.subscriptions);
  
	maybeUpdater.onConfigChange().catch(console.error);
  }