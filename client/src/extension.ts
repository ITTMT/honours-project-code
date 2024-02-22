import * as vscode from 'vscode';
import * as path from 'path';
import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
	// context.subscriptions.push(vscode.commands.registerCommand("bhc.open_file", async (file_path: vscode.Uri) => {
	// 	const doc = await vscode.workspace.openTextDocument(
	// 		file_path
	// 	);
	// 	await vscode.window.showTextDocument(doc);
	// }));


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

