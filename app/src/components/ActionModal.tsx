// components/ActionModal.tsx
import React, { useState } from 'react';

import { X } from 'lucide-react';
import { PendingVerificationUser } from '@/utils/interactions/dataGetter';
import { Box, Flex, Heading, Text, Stack, Button, Textarea, Checkbox, Input } from "@chakra-ui/react";

// import * as Checkbox from "@radix-ui/react-checkbox";
import { FaCheck } from "react-icons/fa";

interface Props {
    isOpen: boolean;
    type: 'approve' | 'reject' | 'request_revision' | null;
    user?: PendingVerificationUser;
    onClose: () => void;
    onSubmit: (data: any) => void;
}

export default function ActionModal({ isOpen, type, user, onClose, onSubmit }: Props) {
    const [notes, setNotes] = useState('');
    const [reason, setReason] = useState('');
    const [rejectedDocTypes, setRejectedDocTypes] = useState<string[]>([]);
    const [documentId, setDocumentId] = useState('');

    if (!isOpen || !type || !user) return null;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        switch (type) {
            case 'approve':
                onSubmit({ notes, document_id: documentId });
                break;
            case 'reject':
                onSubmit({ reason });
                break;
            case 'request_revision':
                onSubmit({ notes, rejected_document_types: rejectedDocTypes });
                break;
        }
    };

    const getTitle = () => {
        switch (type) {
            case 'approve': return 'Approve User';
            case 'reject': return 'Reject User';
            case 'request_revision': return 'Request Document Revision';
        }
    };


    const handleDocumentIdChange = (value: string) => {
        const cleaned = value.replace(/\s/g, '');
        setDocumentId(cleaned);
    };




    return (
        <Box
            position="fixed"
            inset={0}
            bg="blackAlpha.600"
            display="flex"
            alignItems="center"
            justifyContent="center"
            zIndex={50}
            p={4}
        >
            <Box bg="white" rounded="lg" w="full" maxW="md" overflow="hidden">

                {/* Header */}
                <Flex p={4} borderBottom="1px solid" borderColor="gray.200" align="center" justify="space-between">
                    <Heading size="md">{getTitle()}</Heading>
                    <Button
                        onClick={onClose}
                        size="sm"
                        variant="ghost"
                        p={1}
                        _hover={{ bg: "gray.100" }}
                    >
                        <X size={20} />
                    </Button>
                </Flex>

                {/* Form */}
                <Box as="form" onSubmit={handleSubmit} p={4}>
                    <Text mb={4} color="gray.700">
                        {type === "approve" && `Are you sure you want to approve ${user.name}?`}
                        {type === "reject" && `Are you sure you want to reject ${user.name}?`}
                        {type === "request_revision" &&
                            `Request document revisions from ${user.name}`}
                    </Text>



                    {/* Document ID Field - Only for Approve */}
                    {type === "approve" && (
                        <Box mb={4}>
                            <Text fontSize="sm" fontWeight="medium" mb={2}>
                                Document ID:
                            </Text>
                            <Input
                                placeholder="Enter document ID"
                                value={documentId}
                                onChange={(e) => handleDocumentIdChange(e.target.value)}
                            />
                            <Text fontSize="xs" color="gray.500" mt={1}>
                                Reference ID for the document being approved
                            </Text>
                        </Box>
                    )}




                    {/* Request Revision Documents */}
                    {type === "request_revision" && (
                        <Box mb={4}>
                            <Text fontSize="sm" fontWeight="medium" mb={2}>
                                Select documents to reject:
                            </Text>
                            <Stack gap={2} maxH="40" overflowY="auto" border="1px solid" borderColor="gray.200" rounded="md" p={2}>
                                {user.documents.map((doc) => (
                                    <Checkbox.Root
                                        key={doc.id}
                                        onCheckedChange={(checked) => {
                                            if (checked) {
                                                setRejectedDocTypes([...rejectedDocTypes, doc.type]);
                                            } else {
                                                setRejectedDocTypes(rejectedDocTypes.filter((t) => t !== doc.type));
                                            }
                                        }}
                                        className="flex items-center space-x-2"
                                    >
                                        <Checkbox.HiddenInput />
                                        <Checkbox.Control className="w-4 h-4 border rounded flex items-center justify-center border-gray-400">
                                            {rejectedDocTypes.includes(doc.type) && <FaCheck size={12} />}
                                        </Checkbox.Control>
                                        <Checkbox.Label className="text-sm">{doc.type.replace("_", " ")}</Checkbox.Label>
                                    </Checkbox.Root>
                                ))}
                            </Stack>
                        </Box>
                    )}

                    {/* Notes / Reason */}
                    <Box mb={4}>
                        <Text fontSize="sm" fontWeight="medium" mb={2}>
                            {type === "reject" ? "Rejection reason:" : "Notes (optional):"}
                        </Text>
                        <Textarea
                            rows={4}
                            value={type === "reject" ? reason : notes}
                            onChange={(e) => {
                                if (type === "reject") setReason(e.target.value);
                                else setNotes(e.target.value);
                            }}
                            placeholder={
                                type === "reject"
                                    ? "Enter rejection reason..."
                                    : "Add any notes..."
                            }
                            required={type === "reject"}
                        />
                    </Box>

                    {/* Actions */}
                    <Flex justify="flex-end" gap={2}>
                        <Button variant="outline" onClick={onClose}>
                            Cancel
                        </Button>
                        <Button
                            colorScheme={
                                type === "approve" ? "green" : type === "reject" ? "red" : "yellow"
                            }
                            type="submit"
                        >
                            {type === "approve"
                                ? "Approve"
                                : type === "reject"
                                    ? "Reject"
                                    : "Request Revision"}
                        </Button>
                    </Flex>
                </Box>
            </Box>
        </Box>
    );
}



