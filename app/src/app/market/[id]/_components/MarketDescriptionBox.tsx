"use client";

import { useState } from "react";
import { Box, Button, Collapsible, Text } from "@chakra-ui/react";
import { ChevronDown, ChevronUp } from "lucide-react";

interface MarketDescriptionBoxProps {
  description: string;
}

export const MarketDescriptionBox = ({ description }: MarketDescriptionBoxProps) => {
  const [expanded, setExpanded] = useState(false);

  return (
    <Box
      w="full"
      p={4}
      bg="rgba(255,255,255,0.05)"
      borderRadius="lg"
      borderWidth="1px"
      borderColor="white"
      my={4}
    >
      <Collapsible.Root open={expanded} onOpenChange={({ open }) => setExpanded(open)}>
        <Collapsible.Trigger asChild>
          <Button
            variant="ghost"
            w="full"
            justifyContent="space-between"
            onClick={() => setExpanded(!expanded)}
            mb={expanded ? 3 : 0}
          >
            <Text fontWeight="bold">Resolution Criteria</Text>
            {expanded ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
          </Button>
        </Collapsible.Trigger>

        <Collapsible.Content>
          <Text
            color="gray.300"
            lineHeight="tall"
            whiteSpace="pre-wrap"
            wordBreak="break-word"
          >
            {description}
          </Text>
        </Collapsible.Content>
      </Collapsible.Root>

      {/* Preview when collapsed - shows first 2 lines with ellipsis */}
      {!expanded && (
        <Text
          color="gray.400"
          fontSize="sm"
          lineClamp={2}
          mt={2}
          px={2}
        >
          {description}
        </Text>
      )}
    </Box>
  );
};