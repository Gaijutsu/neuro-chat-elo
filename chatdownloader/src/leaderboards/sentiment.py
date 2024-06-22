"""
The Sentiment leaderboard
"""

from typing import Optional

from _types import UserChatPerformance

from .abstractleaderboard import AbstractLeaderboard


class Sentiment(AbstractLeaderboard):
    """
    Leaderboard for most positive and most negative chatters
    """
    @classmethod
    def get_name(cls):
        return 'sentiment'

    def calculate_score(self,
                        performance: UserChatPerformance) -> Optional[float]:
        return performance.metrics['sentiment'] if 'sentiment' in \
            performance.metrics else 0
