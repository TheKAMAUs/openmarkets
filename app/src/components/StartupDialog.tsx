"use client";

import { useEffect, useState } from "react";
import {
    DialogRoot,
    DialogContent,
    DialogHeader,
    DialogBody,
    DialogFooter,
    DialogTitle,
    DialogCloseTrigger,
} from "@/components/ui/dialog";

import { Button } from "@chakra-ui/react";

export default function StartupDialog() {
    const [open, setOpen] = useState(false);

    useEffect(() => {
        setOpen(true); // 🔥 always open on reload
    }, []);

    return (
        <DialogRoot open={open} onOpenChange={(e) => setOpen(e.open)}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>System Restarted</DialogTitle>
                </DialogHeader>

                <DialogCloseTrigger />

                <DialogBody>
                    The app has reloaded. All systems reinitialized.
                </DialogBody>

                <DialogFooter>
                    <Button onClick={() => setOpen(false)}>Close</Button>
                </DialogFooter>
            </DialogContent>
        </DialogRoot>
    );
}