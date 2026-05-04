// components/DocumentViewer.tsx
import React, { useState } from 'react';

import { X, Download, CheckCircle, XCircle } from 'lucide-react';
import { PendingDocument } from '@/utils/interactions/dataGetter';

interface Props {
    documents: PendingDocument[];
    onClose: () => void;
    onSelectForRevision?: (docIds: string[]) => void;
}

export default function DocumentViewer({ documents, onClose, onSelectForRevision }: Props) {
    const [selectedDoc, setSelectedDoc] = useState<PendingDocument | null>(null);
    const [selectedForRevision, setSelectedForRevision] = useState<string[]>([]);

    const toggleDocSelection = (docId: string) => {
        setSelectedForRevision(prev =>
            prev.includes(docId)
                ? prev.filter(id => id !== docId)
                : [...prev, docId]
        );
    };

    const getDocumentTypeLabel = (type: string) => {
        const types: Record<string, string> = {
            passport: 'Passport',
            drivers_license: "Driver's License",
            national_id: 'National ID',
            residence_permit: 'Residence Permit',
            proof_of_address: 'Proof of Address'
        };
        return types[type] || type;
    };

    return (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white rounded-lg w-4/5 max-w-6xl max-h-[90vh] overflow-hidden">
                <div className="p-4 border-b flex items-center justify-between">
                    <h2 className="text-xl font-bold">Verification Documents</h2>
                    <button onClick={onClose} className="p-1 hover:bg-gray-100 rounded">
                        <X size={20} />
                    </button>
                </div>

                <div className="flex h-[calc(90vh-120px)]">
                    {/* Document List */}
                    <div className="w-1/3 border-r overflow-y-auto p-4">
                        <h3 className="font-semibold mb-4">Documents ({documents.length})</h3>
                        <div className="space-y-3">
                            {documents.map((doc) => (
                                <div
                                    key={doc.id}
                                    className={`p-3 border rounded cursor-pointer transition-colors ${selectedDoc?.id === doc.id ? 'border-blue-500 bg-blue-50' : 'hover:bg-gray-50'
                                        } ${onSelectForRevision ? 'flex items-start space-x-2' : ''}`}
                                    onClick={() => setSelectedDoc(doc)}
                                >
                                    {onSelectForRevision && (
                                        <input
                                            type="checkbox"
                                            checked={selectedForRevision.includes(doc.id)}
                                            onChange={() => toggleDocSelection(doc.id)}
                                            className="mt-1"
                                            onClick={(e) => e.stopPropagation()}
                                        />
                                    )}
                                    <div className="flex-1">
                                        <div className="flex items-center justify-between">
                                            <span className="font-medium">{getDocumentTypeLabel(doc.type)}</span>
                                            {doc.status === 'approved' && <CheckCircle size={16} className="text-green-600" />}
                                            {doc.status === 'rejected' && <XCircle size={16} className="text-red-600" />}
                                        </div>
                                        <p className="text-xs text-gray-600 mt-1">
                                            Uploaded: {new Date(doc.uploaded_at).toLocaleDateString()}
                                        </p>
                                        {doc.file_name && (
                                            <p className="text-xs text-gray-500 truncate">{doc.file_name}</p>
                                        )}
                                        {doc.rejection_reason && (
                                            <p className="text-xs text-red-600 mt-1">{doc.rejection_reason}</p>
                                        )}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>

                    {/* Document Preview */}
                    <div className="flex-1 p-4 overflow-y-auto">
                        {selectedDoc ? (
                            <div>
                                <div className="flex items-center justify-between mb-4">
                                    <h3 className="font-semibold">{getDocumentTypeLabel(selectedDoc.type)}</h3>
                                    <a
                                        href={selectedDoc.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="flex items-center space-x-1 text-blue-600 hover:text-blue-800"
                                    >
                                        <Download size={16} />
                                        <span>Download</span>
                                    </a>
                                </div>

                                {/* Document Preview - handles different file types */}
                                <div className="border rounded-lg overflow-hidden bg-gray-50">
                                    {selectedDoc.mime_type?.startsWith('image/') ? (
                                        <img
                                            src={selectedDoc.url}
                                            alt={selectedDoc.type}
                                            className="max-w-full max-h-[60vh] mx-auto"
                                        />
                                    ) : selectedDoc.mime_type === 'application/pdf' ? (
                                        <iframe
                                            src={`${selectedDoc.url}#toolbar=0`}
                                            className="w-full h-[60vh]"
                                            title={selectedDoc.type}
                                        />
                                    ) : (
                                        <div className="p-8 text-center">
                                            <p className="text-gray-600">Preview not available</p>
                                            <a
                                                href={selectedDoc.url}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className="text-blue-600 hover:underline mt-2 inline-block"
                                            >
                                                Click to download
                                            </a>
                                        </div>
                                    )}
                                </div>

                                {selectedDoc.file_name && (
                                    <div className="mt-4 text-sm text-gray-600">
                                        <p>File: {selectedDoc.file_name}</p>
                                        {selectedDoc.file_size && (
                                            <p>Size: {(selectedDoc.file_size / 1024 / 1024).toFixed(2)} MB</p>
                                        )}
                                    </div>
                                )}
                            </div>
                        ) : (
                            <div className="flex items-center justify-center h-full text-gray-500">
                                Select a document to preview
                            </div>
                        )}
                    </div>
                </div>

                {onSelectForRevision && selectedForRevision.length > 0 && (
                    <div className="p-4 border-t bg-gray-50">
                        <button
                            onClick={() => onSelectForRevision(selectedForRevision)}
                            className="px-4 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700"
                        >
                            Request Revision for Selected ({selectedForRevision.length})
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}