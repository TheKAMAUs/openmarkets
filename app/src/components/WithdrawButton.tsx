"use client";

import * as React from "react";
import { Stack, VStack, Button, Icon, Input, } from "@chakra-ui/react";
import { DialogRoot, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogBody, DialogFooter, DialogActionTrigger } from "@/components/ui/dialog";

import { Wallet } from "lucide-react";
import { useForm } from "react-hook-form";
import { useMutation } from "@tanstack/react-query";
import UserService from "@/utils/interactions/dataPosters";
import { toaster } from "./ui/toaster";



type Props = {
    defaultAmount?: number;
    defaultPhone?: string;
};

type FormSchema = {
    amount: number;
    phone: string;
};

export const WithdrawButton = ({
    defaultAmount = 1,
    defaultPhone = "254707433711",
}: Props) => {
    const [open, setOpen] = React.useState(false);

    const { register, handleSubmit, reset } = useForm<FormSchema>({
        defaultValues: {
            amount: defaultAmount,
            phone: defaultPhone,
        },
    });

    const { mutateAsync, isPending } = useMutation({
        mutationFn: (data: FormSchema) =>
            UserService.withdrawFunds({ amount: Number(data.amount), phoneNumber: data.phone }),
    });

    const onSubmit = async (data: FormSchema) => {

        console.log("Payload sent to withdrawFunds:", { amount: data.amount, phoneNumber: data.phone });

        toaster.promise(mutateAsync(data), {
            loading: { title: "Processing withdrawal..." },
            success: () => {
                setOpen(false);
                reset({ amount: defaultAmount, phone: defaultPhone });
                return {
                    title: "Withdrawal request sent!",
                    description: "Check your M-Pesa prompt.",
                };
            },
            error: (error) => ({
                title: "Withdrawal failed",
                description: error instanceof Error ? error.message : "Unknown error",
                closable: true,
            }),
        });
    };

    return (
        <Stack direction="row" gap={4}>
            <DialogRoot open={open} onOpenChange={(e) => setOpen(e.open)} placement="center">
                <DialogTrigger asChild>
                    <Button size="lg" colorScheme="blue">
                        <Icon as={Wallet} mr={2} />
                        Withdraw
                    </Button>
                </DialogTrigger>

                <DialogContent style={{ maxWidth: "400px", padding: "1.5rem" }}>
                    <DialogHeader>
                        <DialogTitle
                            style={{ fontSize: "1.5rem", fontWeight: 600, textAlign: "center" }}
                        >
                            Withdraw Funds
                        </DialogTitle>
                    </DialogHeader>

                    <DialogBody>
                        <form id="withdraw-form" onSubmit={handleSubmit(onSubmit)}>
                            <Stack direction="column" gap={4}>
                                <div>
                                    <label
                                        style={{
                                            display: "block",
                                            marginBottom: "0.5rem",
                                            fontWeight: 500,
                                        }}
                                    >
                                        Phone Number
                                    </label>
                                    <input
                                        type="text"
                                        {...register("phone", { required: true })}
                                        placeholder="Enter phone number"
                                        className="input-field"
                                        style={{
                                            width: "100%",
                                            padding: "0.75rem",
                                            borderRadius: "8px",
                                            border: "1px solid #CBD5E0",
                                            fontSize: "1rem",
                                            outline: "none",
                                        }}
                                    />
                                </div>

                                <div>
                                    <label
                                        style={{
                                            display: "block",
                                            marginBottom: "0.5rem",
                                            fontWeight: 500,
                                        }}
                                    >
                                        Amount
                                    </label>
                                    <input
                                        type="number"
                                        {...register("amount", { required: true, min: 1 })}
                                        placeholder="Enter amount"
                                        className="input-field"
                                        style={{
                                            width: "100%",
                                            padding: "0.75rem",
                                            borderRadius: "8px",
                                            border: "1px solid #CBD5E0",
                                            fontSize: "1rem",
                                            outline: "none",
                                        }}
                                    />
                                </div>
                            </Stack>
                        </form>
                    </DialogBody>

                    <DialogFooter style={{ justifyContent: "space-between", paddingTop: "1.5rem" }}>
                        <DialogActionTrigger asChild>
                            <Button variant="outline">Cancel</Button>
                        </DialogActionTrigger>
                        <Button
                            colorScheme="blue"
                            type="submit"
                            form="withdraw-form"
                            loading={isPending}
                            style={{ minWidth: "120px" }}
                        >
                            Confirm
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </DialogRoot>
        </Stack>
    );
}
