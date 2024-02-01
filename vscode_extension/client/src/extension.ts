import * as vscode from 'vscode';

let cssWindow : vscode.TextDocument; // Only have one window open at a time. Swaps depending on what item is selected.

export function activate(context : vscode.ExtensionContext) {
	console.log('starting');
	
	const myScheme = 'bhc';
	const myProvider = new class implements vscode.TextDocumentContentProvider {
		onDidChangeEmitter = new vscode.EventEmitter<vscode.Uri>();
		onDidChange = this.onDidChangeEmitter.event;

		provideTextDocumentContent(uri: vscode.Uri): string {
			return uri.path;
		}
	};

	context.subscriptions.push(vscode.workspace.registerTextDocumentContentProvider(myScheme, myProvider));

	context.subscriptions.push(vscode.commands.registerCommand('bhc.testing123', async () => {
		await vscode.window.showInputBox({ placeHolder: "test..."}).then( x => {
			vscode.workspace.openTextDocument({
				content: x,
				language: 'css'
			}).then (y => {
				vscode.window.showTextDocument(y, {preview: false});
			});
		});
	}));
}

export function deactivate() {}