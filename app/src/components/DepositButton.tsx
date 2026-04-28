"use client";

import * as React from "react";
import { Button, Icon, Stack } from "@chakra-ui/react";
import { DialogRoot, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogBody, DialogFooter, DialogActionTrigger } from "@/components/ui/dialog";
import { Wallet } from "lucide-react";
import { useForm } from "react-hook-form";
import { useMutation } from "@tanstack/react-query";
import UserService from "@/utils/interactions/dataPosters";
import { toaster } from "./ui/toaster";

type Props = {
    defaultAmount?: number;
    defaultPhone?: string;
    defaultReference?: string;
};

type FormSchema = {
    amount: number;
    phone: string;
    accountReference: string;
};

export const DepositButton = ({
    defaultAmount = 1,
    defaultPhone = "254707433711",
    defaultReference = "1234",
}: Props) => {
    const [open, setOpen] = React.useState(false);

    const { register, handleSubmit, reset } = useForm<FormSchema>({
        defaultValues: {
            amount: defaultAmount,
            phone: defaultPhone,
            accountReference: defaultReference,
        },
    });

    const { mutateAsync, isPending } = useMutation({
        mutationFn: (data: FormSchema) =>
            UserService.depositFunds({
                amount: data.amount,
                phone_number: data.phone,
                account_reference: data.accountReference,
            }),
    });

    const onSubmit = async (data: FormSchema) => {
        toaster.promise(
            mutateAsync(data),
            {
                loading: { title: "Processing deposit..." },
                success: () => {
                    setOpen(false);
                    reset({ amount: defaultAmount, phone: defaultPhone, accountReference: defaultReference });
                    return { title: "Deposit request sent!", description: "Check your M-Pesa prompt." };
                },
                error: (error) => ({
                    title: "Deposit failed",
                    description: error instanceof Error ? error.message : "Unknown error",
                    closable: true,
                }),
            }
        );
    };

    return (
        <Stack direction="row" gap="4">
            <DialogRoot open={open} onOpenChange={(e) => setOpen(e.open)} placement="center">
                <DialogTrigger asChild>
                    <Button size="lg" colorScheme="blue">
                        <Icon>
                            <Wallet />
                        </Icon>
                        Deposit Funds
                    </Button>
                </DialogTrigger>

                <DialogContent style={{ maxWidth: "400px", padding: "1.5rem" }}>
                    <DialogHeader>
                        <DialogTitle style={{ fontSize: "1.5rem", fontWeight: 600, textAlign: "center" }}>
                            Deposit Funds
                        </DialogTitle>
                    </DialogHeader>

                    <DialogBody>
                        <form id="deposit-form" onSubmit={handleSubmit(onSubmit)}>
                            <Stack direction="column" gap={4}>
                                <div>
                                    <label style={{ display: "block", marginBottom: "0.5rem", fontWeight: 500 }}>
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
                                    <label style={{ display: "block", marginBottom: "0.5rem", fontWeight: 500 }}>
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

                                <div>
                                    <label style={{ display: "block", marginBottom: "0.5rem", fontWeight: 500 }}>
                                        Account Reference
                                    </label>
                                    <input
                                        type="text"
                                        {...register("accountReference", { required: true })}
                                        placeholder="Enter account reference"
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
                            form="deposit-form"
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

};
