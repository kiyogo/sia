import React, { useState, useEffect } from 'react';
import { WorkspaceFileInfo } from '../types';
import { invokeCommand } from '../lib/tauriBridge';
import {
  FolderTree,
  FileText,
  Folder,
  Save,
  RefreshCw,
  ShieldCheck,
  FileCode,
  Check,
} from 'lucide-react';

export const Workspace: React.FC = () => {
  const [workspacePath, setWorkspacePath] = useState('');
  const [files, setFiles] = useState<WorkspaceFileInfo[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string>('');
  const [isSaving, setIsSaving] = useState(false);
  const [savedNotice, setSavedNotice] = useState(false);

  useEffect(() => {
    loadWorkspace();
  }, []);

  useEffect(() => {
    if (selectedFile) {
      loadFileContent(selectedFile);
    }
  }, [selectedFile]);

  const loadWorkspace = async () => {
    try {
      const p = await invokeCommand<string>('get_workspace_path');
      setWorkspacePath(p);
      const list = await invokeCommand<WorkspaceFileInfo[]>('list_workspace_files');
      setFiles(list);
    } catch (e) {
      console.error('Failed to load workspace', e);
    }
  };

  const loadFileContent = async (relPath: string) => {
    try {
      const content = await invokeCommand<string>('read_workspace_file', { filePath: relPath });
      setFileContent(content);
    } catch (e) {
      console.error('Failed to read file', e);
      setFileContent(`[Error reading file: ${e}]`);
    }
  };

  const handleSaveFile = async () => {
    if (!selectedFile) return;
    setIsSaving(true);
    try {
      await invokeCommand('write_workspace_file', {
        filePath: selectedFile,
        content: fileContent,
      });
      setSavedNotice(true);
      setTimeout(() => setSavedNotice(false), 2000);
    } catch (e) {
      console.error('Failed to save file', e);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex-1 flex h-full overflow-hidden">
      {/* File Tree Column */}
      <div className="w-72 border-r border-slate-800 bg-slate-950/70 flex flex-col justify-between">
        <div>
          <div className="p-4 border-b border-slate-800 space-y-2">
            <div className="flex items-center justify-between">
              <span className="font-semibold text-xs text-slate-200 flex items-center gap-1.5">
                <FolderTree className="w-4 h-4 text-indigo-400" />
                <span>Files Explorer</span>
              </span>
              <button
                onClick={loadWorkspace}
                title="Refresh Files"
                className="text-slate-400 hover:text-slate-200 p-1"
              >
                <RefreshCw className="w-3.5 h-3.5" />
              </button>
            </div>
            <div className="p-2 bg-slate-900 rounded font-mono text-[10px] text-slate-400 truncate select-all">
              {workspacePath}
            </div>
          </div>

          <div className="p-2 overflow-y-auto space-y-0.5">
            {files.length === 0 ? (
              <div className="p-6 text-center text-xs text-slate-500">Workspace is empty</div>
            ) : (
              files.map((f) => {
                const isSelected = selectedFile === f.path;
                return (
                  <button
                    key={f.path}
                    onClick={() => {
                      if (!f.is_dir) setSelectedFile(f.path);
                    }}
                    className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs transition truncate ${
                      isSelected
                        ? 'bg-indigo-600/20 text-indigo-300 font-medium border border-indigo-500/30'
                        : 'text-slate-400 hover:bg-slate-900 hover:text-slate-200'
                    }`}
                  >
                    {f.is_dir ? (
                      <Folder className="w-3.5 h-3.5 text-indigo-400 flex-shrink-0" />
                    ) : (
                      <FileCode className="w-3.5 h-3.5 text-slate-400 flex-shrink-0" />
                    )}
                    <span className="truncate">{f.name}</span>
                  </button>
                );
              })
            )}
          </div>
        </div>

        {/* Sandbox Notice */}
        <div className="p-3 border-t border-slate-800 text-[11px] text-slate-400 flex items-center gap-2">
          <ShieldCheck className="w-4 h-4 text-emerald-400 flex-shrink-0" />
          <span>Workspace Scoped Sandbox Active</span>
        </div>
      </div>

      {/* File Editor View */}
      <div className="flex-1 flex flex-col bg-slate-950 overflow-hidden">
        {selectedFile ? (
          <>
            <div className="p-3 border-b border-slate-800 bg-slate-950 flex items-center justify-between">
              <span className="font-mono text-xs text-slate-300 flex items-center gap-2">
                <FileText className="w-4 h-4 text-indigo-400" />
                {selectedFile}
              </span>

              <div className="flex items-center gap-2">
                {savedNotice && (
                  <span className="text-xs text-emerald-400 flex items-center gap-1 font-medium">
                    <Check className="w-3.5 h-3.5" />
                    Saved
                  </span>
                )}
                <button
                  onClick={handleSaveFile}
                  disabled={isSaving}
                  className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs flex items-center gap-1.5 transition shadow"
                >
                  <Save className="w-3.5 h-3.5" />
                  <span>{isSaving ? 'Saving...' : 'Save File'}</span>
                </button>
              </div>
            </div>

            <textarea
              value={fileContent}
              onChange={(e) => setFileContent(e.target.value)}
              className="flex-1 p-4 bg-slate-950 text-slate-200 font-mono text-xs leading-relaxed resize-none focus:outline-none select-text"
              spellCheck={false}
            />
          </>
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-slate-500 text-xs">
            <FileCode className="w-10 h-10 text-slate-600 mb-2" />
            <span>Select a file from the workspace explorer to view or edit</span>
          </div>
        )}
      </div>
    </div>
  );
};
