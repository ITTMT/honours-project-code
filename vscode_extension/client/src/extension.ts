import * as vscode from 'vscode';

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
		const what = await vscode.window.showInputBox({ placeHolder: 'test...'});
		if (what) { 
			const uri = vscode.Uri.parse('test:' + what);
			const doc = await vscode.workspace.openTextDocument({
				content: uri,
				language: 'css'
			}
			);
			await vscode.window.showTextDocument(doc, { preview: false});
		}
	}));

	console.log('started');
}

export function deactivate() {}