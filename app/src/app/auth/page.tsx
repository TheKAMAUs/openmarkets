"use client";

import { useState } from 'react';
import {
    createUserWithEmailAndPassword,
    signInWithEmailAndPassword,
    sendEmailVerification,
    GoogleAuthProvider,
    signInWithPopup,
    sendPasswordResetEmail
} from 'firebase/auth';

import { useRouter } from 'next/navigation';
import { auth } from '../lib/fireConfig';
import { useMutation } from '@tanstack/react-query';
import useRevalidation from '@/hooks/useRevalidate';
import { UserAuthActions } from '@/utils/interactions/dataPosters';
import { toaster } from '@/components/ui/toaster';
import cookie from "js-cookie";


export default function AuthPage() {
    const [email, setEmail] = useState('');
    const [referralCode, setReferralCode] = useState('');
    const [password, setPassword] = useState('');
    const [isLogin, setIsLogin] = useState(true);
    const [isForgotPassword, setIsForgotPassword] = useState(false);
    const [loading, setLoading] = useState(false);
    const [resetEmailSent, setResetEmailSent] = useState(false);
    const [error, setError] = useState('');
    const router = useRouter();



    const { mutateAsync } = useMutation({
        mutationFn: UserAuthActions.handleSignInWithGoogle,
    });
    const revalidate = useRevalidation();

    function handleLoginWithGoogle(idToken: string) {
        toaster.promise(mutateAsync({ id_token: idToken }), {
            error(arg: any) {
                setLoading(false);
                return {
                    title: "Error",
                    description: arg?.message || "Failed to login with google",
                };
            },
            success(arg) {
                cookie.set("polymarketAuthToken", arg.sessionToken, {
                    expires: 60 * 60 * 24 * 30, // 30 days,
                    secure: true,
                });
                queueMicrotask(() => revalidate(["userData"]));
                router.push('/');

                return {
                    title: "Success",
                    description: "Welcome to polymarket",
                };
            },
            loading: {
                title: "Waiting for sign in...",
                description: "Please complete your sign in process in popup window",
            },
        });
    }

    function handleLoginWithEmail(idToken: string) {
        toaster.promise(
            mutateAsync({
                id_token: idToken,
                referralCode: !isLogin && referralCode ? referralCode : undefined
            }), {
            error(arg: any) {
                setLoading(false);
                return {
                    title: "Error",
                    description: arg?.message || "Failed to login",
                };
            },
            success(arg) {
                cookie.set("polymarketAuthToken", arg.sessionToken, {
                    expires: 60 * 60 * 24 * 30, // 30 days,
                    secure: true,
                });
                queueMicrotask(() => revalidate(["userData"]));
                router.push('/');

                return {
                    title: "Success",
                    description: "Welcome to polymarket",
                };
            },
            loading: {
                title: "Signing in...",
                description: "Please wait",
            },
        });
    }

    // Handle Email/Password Auth
    const handleEmailAuth = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError('');
        setResetEmailSent(false);

        try {
            let userCredential;

            if (isLogin) {
                // SIGN IN
                userCredential = await signInWithEmailAndPassword(auth, email, password);
                console.log('✅ Signed in:', userCredential.user.email);

                // 🔥 GET THE ID TOKEN
                const idToken = await userCredential.user.getIdToken();
                console.log('🔑 Firebase ID Token:', idToken);

                // Use the email login handler with toaster
                handleLoginWithEmail(idToken);
            } else {
                // SIGN UP
                userCredential = await createUserWithEmailAndPassword(auth, email, password);
                console.log('✅ Signed up:', userCredential.user.email);

                // Send verification email
                await sendEmailVerification(userCredential.user);
                toaster.success({
                    title: "Verification email sent",
                    description: "Please check your inbox",
                });

                // 🔥 GET THE ID TOKEN
                const idToken = await userCredential.user.getIdToken();
                console.log('🔑 Firebase ID Token:', idToken);

                // Use the email login handler with toaster
                handleLoginWithEmail(idToken);
            }

        } catch (error: any) {
            console.error('Auth error:', error);

            // Handle specific Firebase errors
            if (error.code === 'auth/user-not-found') {
                setError('No account found with this email. Please sign up first.');
            } else if (error.code === 'auth/wrong-password') {
                setError('Incorrect password. Please try again.');
            } else if (error.code === 'auth/email-already-in-use') {
                setError('An account already exists with this email. Please sign in.');
            } else if (error.code === 'auth/weak-password') {
                setError('Password is too weak. Please use at least 6 characters.');
            } else {
                setError(error.message);
            }

            setLoading(false);
            toaster.error({
                title: "Authentication Error",
                description: error.message,
            });
        }
    };

    // Handle Google Sign-In
    const handleGoogleSignIn = async () => {
        setLoading(true);
        setError('');

        try {
            const provider = new GoogleAuthProvider();
            const userCredential = await signInWithPopup(auth, provider);

            console.log('✅ Google sign-in:', userCredential.user.email);

            // 🔥 GET THE ID TOKEN
            const idToken = await userCredential.user.getIdToken();
            console.log('🔑 Firebase ID Token:', idToken);

            // Use the Google login handler with toaster
            handleLoginWithGoogle(idToken);

        } catch (error: any) {
            console.error('Google sign-in error:', error);
            setError(error.message);
            setLoading(false);
            toaster.error({
                title: "Google Sign-In Error",
                description: error.message,
            });
        }
    };

    // Handle Forgot Password
    const handleForgotPassword = async (e: React.FormEvent) => {
        e.preventDefault();

        if (!email) {
            setError('Please enter your email address');
            return;
        }

        setLoading(true);
        setError('');
        setResetEmailSent(false);

        try {
            await sendPasswordResetEmail(auth, email);
            setResetEmailSent(true);
            toaster.success({
                title: "Password Reset Email Sent",
                description: "Check your inbox for instructions to reset your password.",
            });
        } catch (error: any) {
            console.error('Password reset error:', error);

            if (error.code === 'auth/user-not-found') {
                setError('No account found with this email address.');
            } else if (error.code === 'auth/invalid-email') {
                setError('Please enter a valid email address.');
            } else {
                setError(error.message);
            }

            toaster.error({
                title: "Password Reset Error",
                description: error.message,
            });
        } finally {
            setLoading(false);
        }
    };

    // Go back to login
    const handleBackToLogin = () => {
        setIsForgotPassword(false);
        setResetEmailSent(false);
        setError('');
    };

    return (
        <div style={{
            minHeight: '100vh',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: '#f3f4f6'
        }}>
            <div style={{
                backgroundColor: 'white',
                padding: '2rem',
                borderRadius: '0.5rem',
                boxShadow: '0 4px 6px rgba(0,0,0,0.1)',
                width: '100%',
                maxWidth: '400px'
            }}>
                <h1 style={{ fontSize: '1.5rem', fontWeight: 'bold', marginBottom: '1.5rem', textAlign: 'center' }}>
                    {isForgotPassword
                        ? 'Reset Password'
                        : (isLogin ? 'Sign In' : 'Sign Up')}
                </h1>

                {error && (
                    <div style={{
                        backgroundColor: '#fee2e2',
                        color: '#dc2626',
                        padding: '0.75rem',
                        borderRadius: '0.375rem',
                        marginBottom: '1rem',
                        fontSize: '0.875rem'
                    }}>
                        {error}
                    </div>
                )}

                {resetEmailSent && (
                    <div style={{
                        backgroundColor: '#dcfce7',
                        color: '#166534',
                        padding: '1rem',
                        borderRadius: '0.375rem',
                        marginBottom: '1rem',
                        fontSize: '0.875rem'
                    }}>
                        <p style={{ fontWeight: 'bold', marginBottom: '0.25rem' }}>✓ Reset email sent!</p>
                        <p>Check your inbox for instructions to reset your password.</p>
                    </div>
                )}

                {/* Forgot Password Form */}
                {isForgotPassword ? (
                    <form onSubmit={handleForgotPassword} style={{ marginBottom: '1rem' }}>
                        <div style={{ marginBottom: '1.5rem' }}>
                            <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: '500', marginBottom: '0.25rem' }}>
                                Email Address
                            </label>
                            <input
                                type="email"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                required
                                placeholder="Enter your email"
                                style={{
                                    width: '100%',
                                    padding: '0.5rem',
                                    border: '1px solid #d1d5db',
                                    borderRadius: '0.375rem',
                                    fontSize: '0.875rem'
                                }}
                            />
                        </div>

                        <button
                            type="submit"
                            disabled={loading}
                            style={{
                                width: '100%',
                                backgroundColor: '#3b82f6',
                                color: 'white',
                                padding: '0.5rem',
                                borderRadius: '0.375rem',
                                fontWeight: '500',
                                cursor: loading ? 'not-allowed' : 'pointer',
                                opacity: loading ? 0.7 : 1,
                                border: 'none',
                                marginBottom: '0.75rem'
                            }}
                        >
                            {loading ? 'Sending...' : 'Send Reset Email'}
                        </button>

                        <button
                            type="button"
                            onClick={handleBackToLogin}
                            style={{
                                width: '100%',
                                background: 'none',
                                border: 'none',
                                color: '#6b7280',
                                fontSize: '0.875rem',
                                cursor: 'pointer',
                                textDecoration: 'underline'
                            }}
                        >
                            Back to Login
                        </button>
                    </form>
                ) : (
                    <>
                        {/* Email/Password Form */}
                        <form onSubmit={handleEmailAuth} style={{ marginBottom: '1rem' }}>
                            <div style={{ marginBottom: '1rem' }}>
                                <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: '500', marginBottom: '0.25rem' }}>
                                    Email
                                </label>
                                <input
                                    type="email"
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    required
                                    style={{
                                        width: '100%',
                                        padding: '0.5rem',
                                        border: '1px solid #d1d5db',
                                        borderRadius: '0.375rem',
                                        fontSize: '0.875rem'
                                    }}
                                />
                            </div>

                            <div style={{ marginBottom: '1rem' }}>
                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.25rem' }}>
                                    <label style={{ fontSize: '0.875rem', fontWeight: '500' }}>
                                        Password
                                    </label>
                                    {isLogin && (
                                        <button
                                            type="button"
                                            onClick={() => setIsForgotPassword(true)}
                                            style={{
                                                background: 'none',
                                                border: 'none',
                                                color: '#3b82f6',
                                                fontSize: '0.75rem',
                                                cursor: 'pointer',
                                                textDecoration: 'underline'
                                            }}
                                        >
                                            Forgot password?
                                        </button>
                                    )}
                                </div>
                                <input
                                    type="password"
                                    value={password}
                                    onChange={(e) => setPassword(e.target.value)}
                                    required
                                    minLength={6}
                                    style={{
                                        width: '100%',
                                        padding: '0.5rem',
                                        border: '1px solid #d1d5db',
                                        borderRadius: '0.375rem',
                                        fontSize: '0.875rem'
                                    }}
                                />
                                {!isLogin && (
                                    <p style={{ fontSize: '0.75rem', color: '#6b7280', marginTop: '0.25rem' }}>
                                        Password must be at least 6 characters
                                    </p>
                                )}
                            </div>

                            {!isLogin && (
                                <div style={{ marginBottom: '1rem' }}>
                                    <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: '500', marginBottom: '0.25rem' }}>
                                        Referral Code (optional)
                                    </label>
                                    <input
                                        type="text"
                                        value={referralCode}
                                        onChange={(e) => setReferralCode(e.target.value)}
                                        placeholder="Enter referral code"
                                        style={{
                                            width: '100%',
                                            padding: '0.5rem',
                                            border: '1px solid #d1d5db',
                                            borderRadius: '0.375rem',
                                            fontSize: '0.875rem'
                                        }}
                                    />
                                </div>
                            )}


                            <button
                                type="submit"
                                disabled={loading}
                                style={{
                                    width: '100%',
                                    backgroundColor: '#3b82f6',
                                    color: 'white',
                                    padding: '0.5rem',
                                    borderRadius: '0.375rem',
                                    fontWeight: '500',
                                    cursor: loading ? 'not-allowed' : 'pointer',
                                    opacity: loading ? 0.7 : 1,
                                    border: 'none',
                                    marginBottom: '0.75rem'
                                }}
                            >
                                {loading ? 'Processing...' : (isLogin ? 'Sign In' : 'Sign Up')}
                            </button>
                        </form>

                        {/* Google Sign-In Button */}
                        <button
                            onClick={handleGoogleSignIn}
                            disabled={loading}
                            style={{
                                width: '100%',
                                backgroundColor: 'white',
                                color: '#374151',
                                padding: '0.5rem',
                                borderRadius: '0.375rem',
                                fontWeight: '500',
                                cursor: loading ? 'not-allowed' : 'pointer',
                                opacity: loading ? 0.7 : 1,
                                border: '1px solid #d1d5db',
                                marginBottom: '1rem',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                gap: '0.5rem'
                            }}
                        >
                            <svg width="20" height="20" viewBox="0 0 24 24">
                                <path
                                    fill="#4285F4"
                                    d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
                                />
                                <path
                                    fill="#34A853"
                                    d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
                                />
                                <path
                                    fill="#FBBC05"
                                    d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
                                />
                                <path
                                    fill="#EA4335"
                                    d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
                                />
                            </svg>
                            Sign in with Google
                        </button>

                        {/* Toggle between Login/Signup */}
                        <p style={{ textAlign: 'center', fontSize: '0.875rem', color: '#6b7280' }}>
                            {isLogin ? "Don't have an account? " : "Already have an account? "}
                            <button
                                onClick={() => setIsLogin(!isLogin)}
                                style={{
                                    color: '#3b82f6',
                                    background: 'none',
                                    border: 'none',
                                    cursor: 'pointer',
                                    textDecoration: 'underline'
                                }}
                            >
                                {isLogin ? 'Sign Up' : 'Sign In'}
                            </button>
                        </p>
                    </>
                )}
            </div>
        </div>
    );
}