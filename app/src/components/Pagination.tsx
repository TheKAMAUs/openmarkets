// components/Pagination.tsx
import React from 'react';

import { ChevronLeft, ChevronRight } from 'lucide-react';
import { PaginationMetadata } from '@/utils/interactions/dataGetter';
import { Box, HStack, Button, Text } from "@chakra-ui/react";

interface Props {
    pagination: PaginationMetadata;
    onPageChange: (newOffset: number) => void;
}

export default function Pagination({ pagination, onPageChange }: Props) {
    const { total, limit, offset, has_more } = pagination;
    const currentPage = Math.floor(offset / limit) + 1;
    const totalPages = Math.ceil(total / limit);

    const goToPage = (page: number) => {
        const newOffset = (page - 1) * limit;
        onPageChange(newOffset);
    };

    return (
        <Box mt={8} borderTop="1px solid" borderColor="gray.200" pt={4}>
            <HStack justify="space-between">
                {/* Result Info */}
                <Text fontSize="sm" color="gray.600">
                    Showing {offset + 1} to {Math.min(offset + limit, total)} of {total} results
                </Text>

                {/* Page Buttons */}
                <HStack gap={2}>
                    {/* Previous Button */}
                    <Button
                        onClick={() => goToPage(currentPage - 1)}
                        disabled={currentPage === 1}
                        size="sm"
                        variant="outline"
                    >
                        <HStack gap={1}>
                            <ChevronLeft size={16} />
                            <span>Prev</span>
                        </HStack>
                    </Button>


                    {/* Page Numbers */}
                    {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                        let pageNum;
                        if (totalPages <= 5) pageNum = i + 1;
                        else if (currentPage <= 3) pageNum = i + 1;
                        else if (currentPage >= totalPages - 2) pageNum = totalPages - 4 + i;
                        else pageNum = currentPage - 2 + i;

                        return (
                            <Button
                                key={pageNum}
                                onClick={() => goToPage(pageNum)}
                                size="sm"
                                variant={currentPage === pageNum ? "solid" : "outline"}
                                colorScheme={currentPage === pageNum ? "blue" : "gray"}
                                minW="8"
                            >
                                {pageNum}
                            </Button>
                        );
                    })}

                    {/* Next Button */}
                    <Button
                        onClick={() => goToPage(currentPage + 1)}
                        disabled={!has_more}
                        size="sm"
                        variant="outline"
                    >
                        <HStack gap={1}>
                            <span>Next</span>
                            <ChevronRight size={16} />
                        </HStack>
                    </Button>
                </HStack>
            </HStack>
        </Box>
    );
}