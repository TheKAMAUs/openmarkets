// components/UserVerificationCard.tsx
import React from 'react';

import { formatDistanceToNow, format } from 'date-fns';
import { PendingVerificationUser } from '@/utils/interactions/dataGetter';
import {
    Card,
    Flex,
    Avatar,
    Heading,
    Text,
    Badge,
    Grid,
    Box,
    Button,
    Stack,
    Icon
} from "@chakra-ui/react"

import {
    FaUser,
    FaEnvelope,
    FaClock,
    FaFileAlt,
    FaCheckCircle,
    FaHourglassHalf,
    FaEye,
    FaEdit,
    FaTimes,
    FaCheck
} from "react-icons/fa"


interface Props {
    user: PendingVerificationUser;
    onViewDetails: () => void;
    onApprove: () => void;
    onReject: () => void;
    onRequestRevision: () => void;
}

export default function UserVerificationCard({
    user,
    onViewDetails,
    onApprove,
    onReject,
    onRequestRevision
}: Props) {
    const getStatusBadge = () => {
        if (user.pending_documents === user.total_documents) {
            return <span className="px-2 py-1 bg-yellow-100 text-yellow-800 text-xs rounded-full">All Pending</span>;
        } else if (user.approved_documents > 0) {
            return <span className="px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full">Partial Approved</span>;
        }
        return null;
    };

    return (
        <Card.Root
            shadow="sm"
            borderWidth="1px"
            borderColor="gray.200"
            _hover={{ shadow: "lg", transform: "translateY(-2px)" }}
            transition="all 0.2s"
            borderRadius="xl"
        >
            <Card.Body>

                {/* Header */}
                <Flex justify="space-between" align="start">

                    <Flex gap={4} align="center">

                        <Avatar.Root size="lg">
                            <Avatar.Image src={user.avatar || "/default-avatar.png"} />
                            <Avatar.Fallback name={user.name} />
                        </Avatar.Root>

                        <Box>

                            <Flex align="center" gap={2}>
                                <Icon as={FaUser} color="gray.500" />
                                <Heading size="sm">{user.name}</Heading>
                            </Flex>

                            <Flex align="center" gap={2} color="gray.500" mt={1}>
                                <Icon as={FaEnvelope} />
                                <Text fontSize="sm">{user.email}</Text>
                            </Flex>

                        </Box>

                    </Flex>

                    <Flex align="center" gap={2}>

                        {getStatusBadge()}

                        <Badge
                            colorPalette="blue"
                            borderRadius="full"
                            px={3}
                            py={1}
                            fontWeight="medium"
                        >
                            <Flex align="center" gap={1}>
                                <Icon as={FaClock} />
                                Pending {user.days_pending} days
                            </Flex>
                        </Badge>

                    </Flex>

                </Flex>

                {/* Stats */}
                <Grid templateColumns="repeat(4,1fr)" gap={4} mt={6}>

                    <Box
                        textAlign="center"
                        p={3}
                        borderRadius="lg"
                        bg="gray.50"
                    >
                        <Flex justify="center" mb={1}>
                            <Icon as={FaFileAlt} color="gray.600" boxSize={5} />
                        </Flex>
                        <Text fontWeight="bold" fontSize="2xl">
                            {user.total_documents}
                        </Text>
                        <Text fontSize="xs" color="gray.500">
                            Total Docs
                        </Text>
                    </Box>

                    <Box
                        textAlign="center"
                        p={3}
                        borderRadius="lg"
                        bg="orange.50"
                    >
                        <Flex justify="center" mb={1}>
                            <Icon as={FaHourglassHalf} color="orange.500" boxSize={5} />
                        </Flex>
                        <Text fontWeight="bold" fontSize="2xl" color="orange.500">
                            {user.pending_documents}
                        </Text>
                        <Text fontSize="xs" color="gray.500">
                            Pending
                        </Text>
                    </Box>

                    <Box
                        textAlign="center"
                        p={3}
                        borderRadius="lg"
                        bg="green.50"
                    >
                        <Flex justify="center" mb={1}>
                            <Icon as={FaCheckCircle} color="green.500" boxSize={5} />
                        </Flex>
                        <Text fontWeight="bold" fontSize="2xl" color="green.500">
                            {user.approved_documents}
                        </Text>
                        <Text fontSize="xs" color="gray.500">
                            Approved
                        </Text>
                    </Box>

                    <Box
                        textAlign="center"
                        p={3}
                        borderRadius="lg"
                        bg="gray.50"
                    >
                        <Flex justify="center" mb={1}>
                            <Icon as={FaFileAlt} color="gray.600" boxSize={5} />
                        </Flex>
                        <Text fontWeight="bold" fontSize="2xl">
                            {user.documents.length}
                        </Text>
                        <Text fontSize="xs" color="gray.500">
                            Uploaded
                        </Text>
                    </Box>

                </Grid>

                {/* Notes */}
                {user.verification_notes && (
                    <Box
                        mt={5}
                        p={4}
                        bg="gray.50"
                        borderRadius="lg"
                        borderWidth="1px"
                        borderColor="gray.200"
                    >
                        <Text fontSize="sm">
                            <strong>Notes:</strong> {user.verification_notes}
                        </Text>
                    </Box>
                )}

                {/* Footer */}
                <Flex justify="space-between" align="center" mt={6}>

                    <Text fontSize="sm" color="gray.500">
                        Applied{" "}
                        {user.verification_applied_at
                            ? format(new Date(user.verification_applied_at), "MMM d, yyyy")
                            : "N/A"}
                    </Text>

                    <Stack direction="row">

                        <Button size="sm" variant="outline" onClick={onViewDetails}>
                            <Icon as={FaEye} mr={1} />
                            View
                        </Button>

                        <Button
                            size="sm"
                            colorPalette="yellow"
                            variant="subtle"
                            onClick={onRequestRevision}
                        >
                            <Icon as={FaEdit} mr={1} />
                            Revision
                        </Button>

                        <Button
                            size="sm"
                            colorPalette="red"
                            variant="subtle"
                            onClick={onReject}
                        >
                            <Icon as={FaTimes} mr={1} />
                            Reject
                        </Button>

                        <Button
                            size="sm"
                            colorPalette="green"
                            onClick={onApprove}
                        >
                            <Icon as={FaCheck} mr={1} />
                            Approve
                        </Button>

                    </Stack>

                </Flex>

            </Card.Body>
        </Card.Root>
    );
}