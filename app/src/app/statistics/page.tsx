"use client";

import { ProfitLossAPI, UserProfitRanking } from '@/utils/interactions/dataGetter';
import { useEffect, useState } from 'react';

export default function LeaderboardPage() {
    const [winners, setWinners] = useState<UserProfitRanking[]>([]);
    const [losers, setLosers] = useState<UserProfitRanking[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchLeaderboard = async () => {
            try {
                const data = await ProfitLossAPI.getLeaderboard();
                setWinners(data.winners);
                setLosers(data.losers);
            } catch (error) {
                console.error('Failed to fetch leaderboard:', error);
            } finally {
                setLoading(false);
            }
        };

        fetchLeaderboard();
    }, []);

    return (
        <div style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto' }}>
            <h1 style={{ fontSize: '2rem', marginBottom: '30px' }}>
                Leaderboard
            </h1>

            {loading ? (
                <p>Loading...</p>
            ) : (
                <div style={{
                    display: 'grid',
                    gridTemplateColumns: '1fr 1fr',
                    gap: '30px'
                }}>
                    {/* =======================
                        WINNERS
                    ======================= */}
                    <section style={{
                        background: '#f8f9fa',
                        padding: '20px',
                        borderRadius: '10px'
                    }}>
                        <h2 style={{ color: '#28a745', marginBottom: '20px' }}>
                            🏆 Top Winners
                        </h2>

                        {winners.map((winner, index) => (
                            <div key={winner.user_id} style={{
                                padding: '15px',
                                marginBottom: '10px',
                                background: 'white',
                                borderRadius: '8px',
                                boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
                            }}>
                                <div style={{
                                    display: 'flex',
                                    alignItems: 'center',
                                    gap: '15px'
                                }}>
                                    <span style={{
                                        width: '30px',
                                        height: '30px',
                                        background: '#28a745',
                                        color: 'white',
                                        borderRadius: '50%',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontWeight: 'bold'
                                    }}>
                                        {index + 1}
                                    </span>

                                    <div>
                                        <p style={{ fontWeight: 'bold', margin: 0 }}>
                                            {winner.name}
                                        </p>

                                        <p style={{
                                            color: '#28a745',
                                            fontWeight: 'bold',
                                            margin: '5px 0 0 0'
                                        }}>
                                            +${winner.net_profit}
                                        </p>

                                        <p style={{
                                            fontSize: '0.9rem',
                                            color: '#666',
                                            margin: '5px 0 0 0'
                                        }}>
                                            Win Rate: {winner.win_rate.toFixed(1)}%
                                        </p>
                                    </div>
                                </div>
                            </div>
                        ))}
                    </section>

                    {/* =======================
                        LOSERS
                    ======================= */}
                    <section style={{
                        background: '#f8f9fa',
                        padding: '20px',
                        borderRadius: '10px'
                    }}>
                        <h2 style={{ color: '#dc3545', marginBottom: '20px' }}>
                            📉 Top Losers
                        </h2>

                        {losers.map((loser, index) => (
                            <div key={loser.user_id} style={{
                                padding: '15px',
                                marginBottom: '10px',
                                background: 'white',
                                borderRadius: '8px',
                                boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
                            }}>
                                <div style={{
                                    display: 'flex',
                                    alignItems: 'center',
                                    gap: '15px'
                                }}>
                                    <span style={{
                                        width: '30px',
                                        height: '30px',
                                        background: '#dc3545',
                                        color: 'white',
                                        borderRadius: '50%',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        fontWeight: 'bold'
                                    }}>
                                        {index + 1}
                                    </span>

                                    <div>
                                        <p style={{ fontWeight: 'bold', margin: 0 }}>
                                            {loser.name}
                                        </p>

                                        <p style={{
                                            color: '#dc3545',
                                            fontWeight: 'bold',
                                            margin: '5px 0 0 0'
                                        }}>
                                            -${Math.abs(loser.net_profit)}
                                        </p>

                                        <p style={{
                                            fontSize: '0.9rem',
                                            color: '#666',
                                            margin: '5px 0 0 0'
                                        }}>
                                            Win Rate: {loser.win_rate.toFixed(1)}%
                                        </p>
                                    </div>
                                </div>
                            </div>
                        ))}
                    </section>
                </div>
            )}
        </div>
    );
}