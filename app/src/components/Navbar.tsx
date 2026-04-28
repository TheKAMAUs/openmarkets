// "use client";

// import React, { useEffect, useState } from "react";
// import { Search } from "lucide-react";
// import Image from "next/image";
// import Link from "next/link";
// import { Flex, Input, InputGroup, Text } from "@chakra-ui/react";
// import cookie from "js-cookie";
// import { jwtDecode } from "jwt-decode";

// import NavbarAvatarButton from "./NavbarAvatarButton";
// import NavbarNotificationButton from "./NavbarNotificationButton";

// type SessionClaims = {
//   user_id: string;
//   email?: string;
//   is_admin: boolean;
//   exp: number;
// };

// const Navbar = () => {
//   const [isAdmin, setIsAdmin] = useState(false);

//   useEffect(() => {
//     const token = cookie.get("polymarketAuthToken");
//     if (!token) return;

//     try {
//       const claims: SessionClaims = jwtDecode(token);
//       setIsAdmin(claims.is_admin);
//     } catch (err) {
//       console.error("Failed to decode token:", err);
//     }
//   }, []);

//   return (
//     <Flex
//       padding={4}
//       justifyContent="space-between"
//       borderBottom="1px solid"
//       borderColor="gray.200"
//       alignItems="center"
//     >
//       {/* left side */}
//       <Flex alignItems="center">
//         <Link href="/">
//           <Image src="/assets/logo.svg" alt="Logo" width={135} height={23} />
//         </Link>
//         <Flex as="nav" ml={8} gap={6} display={["none", "flex"]}>
//           {LINKS.map((link) => (
//             <Link href={link.href} key={link.name}>
//               <Text
//                 fontSize="14"
//                 fontWeight="medium"
//                 color="gray.700"
//                 _hover={{ textDecoration: "underline" }}
//               >
//                 {link.name}
//               </Text>
//             </Link>
//           ))}
//         </Flex>
//       </Flex>

//       {/* right section */}
//       <Flex gap={4} alignItems="center">
//         <InputGroup
//           startElement={<Search opacity={0.4} />}
//           display={["none", "flex"]}
//         >
//           <Input placeholder="Search" variant="subtle" />
//         </InputGroup>

//         <NavbarNotificationButton />

//         <Flex direction="column" alignItems="flex-end">
//           <NavbarAvatarButton />

//           {/* Only show admin link if user is admin */}
//           {isAdmin && (
//             <Link href="/admin">
//               <Text
//                 mt={1}
//                 fontSize="6"
//                 fontWeight="medium"
//                 color="blue.600"
//                 _hover={{ textDecoration: "underline" }}
//               >
//                 Admin Dashboard
//               </Text>
//             </Link>
//           )}
//         </Flex>
//       </Flex>
//     </Flex>
//   );
// };

// export default Navbar;

// const LINKS = [
//   { name: "Home", href: "/" },
//   { name: "Profile", href: "/profile" },
// ];






"use client";

import {
  Flex,
  IconButton,
  Input,
  InputGroup,
  Link as ChakraLink,
  Text,
} from "@chakra-ui/react";

import Link from "next/link";
import { useState } from "react";
import { useAuth } from "@/context/AuthContext";

import {
  Home,
  User,
  LogIn,
  Lightbulb,
  Shield,
  Trophy,
  Droplets,
  LayoutDashboard,
} from "lucide-react";

import NavbarNotificationButton from "./NavbarNotificationButton";
import NavbarAvatarButton from "./NavbarAvatarButton";

type SessionClaims = {
  user_id: string;
  email?: string;
  is_admin: boolean;
  exp: number;
};

const Navbar = () => {
  const { isAdmin, isLoading } = useAuth(); // Get auth state from context
  const [adminPanelOpen, setAdminPanelOpen] = useState(false);

  // No more useEffect for token decoding here - it's handled globally!

  // You can show a loading state if needed
  if (isLoading) {
    return <nav>Loading...</nav>; // Or a skeleton loading state
  }

  return (
    <Flex
      padding={4}
      justifyContent="space-between"
      borderBottom="1px solid"
      borderColor="gray.200"
      alignItems="center"
    >
      {/* LEFT */}
      <Flex alignItems="center">
        {/* Logo */}
        <Link href="/">
          <img src="/assets/logo.svg" alt="Logo" width={135} height={23} />
        </Link>

        {/* ICON NAV */}
        {!adminPanelOpen && (
          <Flex ml={6} gap={1} display={["none", "flex"]}>
            {/* Home */}
            <IconButton asChild aria-label="Home" variant="ghost">
              <Link href="/">
                <Home size={18} />
              </Link>
            </IconButton>

            {/* Profile */}
            <IconButton asChild aria-label="Profile" variant="ghost">
              <Link href="/profile">
                <User size={18} />
              </Link>
            </IconButton>

            {/* Auth */}
            <IconButton asChild aria-label="Auth" variant="ghost">
              <Link href="/auth">
                <LogIn size={18} />
              </Link>
            </IconButton>

            {/* Suggestions */}
            <IconButton asChild aria-label="Suggestions" variant="ghost">
              <Link href="/suggestions">
                <Lightbulb size={18} />
              </Link>
            </IconButton>

            {/* Leaderboard */}
            <IconButton asChild aria-label="Leaderboard" variant="ghost">
              <Link href="/leaderboard">
                <Trophy size={18} />
              </Link>
            </IconButton>

            {/* Liquidity */}
            <IconButton asChild aria-label="Liquidity" variant="ghost">
              <Link href="/liquidity">
                <Droplets size={18} />
              </Link>
            </IconButton>
          </Flex>
        )}

        {/* ADMIN MODE NAV */}
        {adminPanelOpen && (
          <Flex ml={6} gap={1} display={["none", "flex"]}>
            <IconButton asChild aria-label="Admin Dashboard" variant="ghost">
              <Link href="/admin">
                <LayoutDashboard size={18} />
              </Link>
            </IconButton>

            <IconButton asChild aria-label="Admin Markets" variant="ghost">
              <Link href="/admin/markets">
                <Shield size={18} />
              </Link>
            </IconButton>

            <IconButton asChild aria-label="Admin Suggestions" variant="ghost">
              <Link href="/admin/suggestions">
                <Lightbulb size={18} />
              </Link>
            </IconButton>

            <IconButton asChild aria-label="Verifications" variant="ghost">
              <Link href="/admin/verifications">
                <Shield size={18} />
              </Link>
            </IconButton>
          </Flex>
        )}
      </Flex>

      {/* RIGHT */}
      <Flex gap={3} alignItems="center">
        {!adminPanelOpen && (
          <InputGroup display={["none", "flex"]}>
            <Input placeholder="Search" variant="subtle" />
          </InputGroup>
        )}

        <NavbarNotificationButton />

        <Flex direction="column" alignItems="flex-end">
          <NavbarAvatarButton />

          {/* ADMIN TOGGLE */}
          {isAdmin && (
            <Text
              mt={1}
              fontSize="10px"
              fontWeight="medium"
              color="blue.600"
              cursor="pointer"
              onClick={() => setAdminPanelOpen((prev) => !prev)}
            >
              {adminPanelOpen ? "Exit Admin" : "Dashboard"}
            </Text>
          )}
        </Flex>
      </Flex>
    </Flex>
  );
};

export default Navbar;

const LINKS = [
  { name: "Home", href: "/" },
  { name: "Profile", href: "/profile" },
  { name: "Verification", href: "/verification" },
];
