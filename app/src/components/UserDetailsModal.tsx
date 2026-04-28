// components/UserDetailsModalSimple.tsx
import React, { useState } from 'react';

import { X, Download, FileText, User, Clock } from 'lucide-react';
import { format } from 'date-fns';
import { PendingVerificationUser } from '@/utils/interactions/dataGetter';
import * as Tabs from "@radix-ui/react-tabs";
import { Box, Flex, Heading, Text, Badge, Stack, Button, Icon } from "@chakra-ui/react";
import {
    FaUser,
    FaEnvelope,
    FaClock,

} from "react-icons/fa";


interface Props {
    user: PendingVerificationUser;
    onClose: () => void;
}

export default function UserDetailsModalSimple({ user, onClose }: Props) {
    const [activeTab, setActiveTab] = useState<'info' | 'documents'>('info');

    return (
        <Box
            position="absolute" // instead of fixed
            top={0}
            left={0}
            right={0}
            bottom={0}
            bg="blackAlpha.600"
            display="flex"
            alignItems="center"
            justifyContent="center"
        >
            <Box
                bg="white"
                borderRadius="xl"
                w="full"
                maxW="3xl"
                maxH="90vh"
                overflow="hidden"
                shadow="lg"
                display="flex"
                flexDirection="column"
            >
                {/* Header */}
                <Flex
                    p={4}
                    borderBottomWidth={1}
                    borderColor="gray.200"
                    justify="space-between"
                    align="center"
                >
                    <Flex align="center" gap={4}>
                        <img
                            src={user.avatar || "/default-avatar.png"}
                            alt={user.name}
                            className="w-12 h-12 rounded-full object-cover"
                        />
                        <Box>
                            <Heading size="md">{user.name}</Heading>
                            <Flex align="center" gap={2} color="gray.500" fontSize="sm">
                                <Icon as={FaEnvelope} />
                                {user.email}
                            </Flex>
                        </Box>
                    </Flex>

                    <Button size="sm" variant="ghost" _hover={{ bg: "gray.100" }} onClick={onClose}>
                        X
                    </Button>
                </Flex>

                {/* Tabs */}
                <Tabs.Root defaultValue="info">
                    <Tabs.List asChild>
                        <Flex borderBottomWidth={1} borderColor="gray.200">
                            <Tabs.Trigger asChild value="info">
                                <Button
                                    flex="1"
                                    variant="ghost"
                                    _selected={{
                                        borderBottom: "2px solid",
                                        borderColor: "blue.500",
                                        color: "blue.500",
                                        fontWeight: "bold",
                                    }}
                                >
                                    <Flex align="center" gap={2}>
                                        <Icon as={FaUser} />
                                        User Info
                                    </Flex>
                                </Button>
                            </Tabs.Trigger>

                            <Tabs.Trigger asChild value="documents">
                                <Button
                                    flex="1"
                                    variant="ghost"
                                    _selected={{
                                        borderBottom: "2px solid",
                                        borderColor: "blue.500",
                                        color: "blue.500",
                                        fontWeight: "bold",
                                    }}
                                >
                                    <Flex align="center" gap={2}>
                                        <Icon as={FaClock} />
                                        Documents ({user.documents.length})
                                    </Flex>
                                </Button>
                            </Tabs.Trigger>
                        </Flex>
                    </Tabs.List>

                    {/* Info Tab */}
                    <Tabs.Content value="info">
                        <Box p={4} overflowY="auto" maxH="calc(90vh - 180px)">
                            <Stack gap={4}>
                                <Flex gap={4} wrap="wrap">
                                    <Box bg="gray.50" p={3} borderRadius="md" flex="1">
                                        <Text fontSize="xs" color="gray.600">Step</Text>
                                        <Text fontWeight="medium">{user.verification_step.replace("_", " ")}</Text>
                                    </Box>
                                    <Box bg="gray.50" p={3} borderRadius="md" flex="1">
                                        <Text fontSize="xs" color="gray.600">Applied</Text>
                                        <Text fontWeight="medium">
                                            {user.verification_applied_at
                                                ? format(new Date(user.verification_applied_at), "MMM d, yyyy")
                                                : "N/A"}
                                        </Text>
                                    </Box>
                                    <Box bg="gray.50" p={3} borderRadius="md" flex="1">
                                        <Text fontSize="xs" color="gray.600">Days Pending</Text>
                                        <Text fontWeight="medium">{user.days_pending} days</Text>
                                    </Box>
                                </Flex>

                                {user.verification_notes && (
                                    <Box bg="yellow.50" p={3} borderRadius="md">
                                        <Text fontSize="sm" fontWeight="medium">Notes</Text>
                                        <Text fontSize="sm">{user.verification_notes}</Text>
                                    </Box>
                                )}

                                <Flex gap={3} mt={2}>
                                    <Box textAlign="center" bg="yellow.50" p={3} borderRadius="md" flex="1">
                                        <Text fontWeight="bold" color="yellow.700">{user.pending_documents}</Text>
                                        <Text fontSize="xs">Pending</Text>
                                    </Box>
                                    <Box textAlign="center" bg="green.50" p={3} borderRadius="md" flex="1">
                                        <Text fontWeight="bold" color="green.700">{user.approved_documents}</Text>
                                        <Text fontSize="xs">Approved</Text>
                                    </Box>
                                    <Box textAlign="center" bg="blue.50" p={3} borderRadius="md" flex="1">
                                        <Text fontWeight="bold" color="blue.700">{user.total_documents}</Text>
                                        <Text fontSize="xs">Total</Text>
                                    </Box>
                                </Flex>
                            </Stack>
                        </Box>
                    </Tabs.Content>

                    {/* Documents Tab */}
                    <Tabs.Content value="documents">
                        <Box p={4} overflowY="auto" maxH="calc(90vh - 180px)">
                            <Stack gap={3}>
                                {user.documents.map((doc) => (
                                    <Box key={doc.id} borderWidth={1} borderColor="gray.200" borderRadius="md" p={2}>
                                        <Text fontSize="xs" color="gray.600" mb={1}>
                                            {doc.type.replace("_", " ")} ({doc.status})
                                        </Text>
                                        <img
                                            src={doc.url}
                                            alt={doc.type}
                                            className="w-full object-cover rounded"
                                        />
                                        {doc.rejection_reason && (
                                            <Text fontSize="xs" color="red.600" mt={1}>
                                                Reason: {doc.rejection_reason}
                                            </Text>
                                        )}
                                    </Box>
                                ))}
                            </Stack>
                        </Box>
                    </Tabs.Content>
                </Tabs.Root>

                {/* Footer */}
                <Box borderBottom="1px solid" borderColor="gray.200" my={2} />
                <Flex justify="flex-end" p={4} bg="gray.50">
                    <Button onClick={onClose} size="sm" colorScheme="gray">
                        Close
                    </Button>
                </Flex>
            </Box>
        </Box>
    );
}