import * as vscode from 'vscode';
import * as path from 'path';
import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
	const serverModule = context.asAbsolutePath(path.join('server', 'target', 'release', 'server.exe'));

	const serverOptions: ServerOptions = {
		run: { module: serverModule, transport: TransportKind.ipc },
		debug: {
			module: serverModule,
			transport: TransportKind.ipc,
		}
	};
	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: 'html' }],
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher('**/.clientrc')
		}
	};

	client = new LanguageClient(
		'languageServerExample',
		'Language Server Example',
		serverOptions,
		clientOptions
	);

	client.start();
}


// export function activate(context : vscode.ExtensionContext) {
// 	console.log('starting');
	
// 	const myScheme = 'bhc';
// 	const myProvider = new class implements vscode.TextDocumentContentProvider {
// 		onDidChangeEmitter = new vscode.EventEmitter<vscode.Uri>();
// 		onDidChange = this.onDidChangeEmitter.event;

// 		provideTextDocumentContent(uri: vscode.Uri): string {
// 			return uri.path;
// 		}
// 	};

// 	context.subscriptions.push(vscode.workspace.registerTextDocumentContentProvider(myScheme, myProvider));

// 	const testa: vscode.Uri = vscode.Uri.file("C:\\Users\\Ollie\\Documents\\CS\\honours-project-code\\css_files\\base.css");

// 	context.subscriptions.push(vscode.commands.registerCommand('bhc.testing123', async () => {
// 		await vscode.workspace.openTextDocument(testa).then (y => {
// 			vscode.window.showTextDocument(y, {preview: false});
// 		});
// 	}));
// }

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}