"""
The metric for measuring the sentiment of chat mesages.

TODO: Add scores for all of the most used emotes in the chat.
      ___         ___                       ___     
     /\  \       /\  \                     /\  \    
    /::\  \     /::\  \       ___          \:\  \   
   /:/\:\__\   /:/\:\  \     /\__\          \:\  \  
  /:/ /:/  /  /:/ /::\  \   /:/__/      _____\:\  \ 
 /:/_/:/  /  /:/_/:/\:\__\ /::\  \     /::::::::\__\ 
 \:\/:/  /   \:\/:/  \/__/ \/\:\  \__  \:\~~\~~\/__/
  \::/__/     \::/__/         \:\/\__\  \:\  \      
   \:\  \      \:\  \          \::/  /   \:\  \     
    \:\__\      \:\__\         /:/  /     \:\__\    
     \/__/       \/__/         \/__/       \/__/    

TODO: Find a better way to measure the overall sentiment of a chatter over a stream.
The current method only takes the average sentiment of all of the messages of a chatter,
which does not account for a chatter sending a single very positive or very negative message.
The metric also needs to provide a higher score for more positive or negative messages.
"""

from typing import Dict, Tuple
import logging
from _types import Comment

from .abstractmetric import AbstractMetric

from vaderSentiment.vaderSentiment import SentimentIntensityAnalyzer

# Sentiment scores for the most used emotes in the chat.
SEVEN_TV_EMOTES  = {
    'Clap': 0.2,
    'OMEGALUL': 0.5,
    'PepeLaugh': 0.5,
    'PogChamp': 0.5,
    'PogU': 0.5,
    'Pog': 0.5,
    'KEKW': 0.5,
    'monkaW': -0.5,
    'Clueless': -0.2,
}

WEIGHT_SENTIMENT = 0.3

class Sentiment(AbstractMetric):
    """
    The chat sentiment metric.
    """
    def __init__(self):
        self.commenter_scores: Dict[str, Tuple[float, int]] = {}
            #Vader
        self.vader_analyser: SentimentIntensityAnalyzer = SentimentIntensityAnalyzer()
        self.vader_analyser.lexicon.update(SEVEN_TV_EMOTES)

    @classmethod  
    def can_parallelize(cls) -> bool:
        return True

    @classmethod
    def get_name(cls) -> str:
        return 'sentiment'

    def get_metric(self, comment: Comment,
                   sequence_no: int) -> list[dict[str, int]]:
        text = ' '.join(
            fragment.text for fragment in comment.message.fragments)

        if len(text) == 0:
            return {}

        sentiment = self.vader_analyser.polarity_scores(text)['compound']

        logging.debug(f"Sentiment for {comment.commenter._id}: {sentiment}")
        
        if comment.commenter._id in self.commenter_scores:
            old_score, old_count = self.commenter_scores[comment.commenter._id]
            self.commenter_scores[comment.commenter._id] = (
                old_score + sentiment, old_count + 1)
        else:
            self.commenter_scores[comment.commenter._id] = (sentiment, 1)

        return {}

    def finish(self) -> Dict[str, float]:
        scores = {}
        for commenter, (score, count) in self.commenter_scores.items():
            scores[commenter] = (score / count) * WEIGHT_SENTIMENT
        return scores
