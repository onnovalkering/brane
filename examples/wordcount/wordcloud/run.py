#!/usr/bin/env python3
import os
import sys
import yaml

from wordcloud import WordCloud
from typing import List
import matplotlib.pyplot as plt

def create(words: List[str], file: str) -> str:
  wordcloud = WordCloud(max_font_size=40).generate(' '.join(words))
  plt.imshow(wordcloud, interpolation='bilinear')
  plt.axis("off")
  plt.savefig(file)

  return file

if __name__ == "__main__":
  command = sys.argv[1]
  argument_file = os.environ["FILE"]
  argument_words = [os.environ[f"WORDS_{i}"] for i in range(int(os.environ["WORDS"]))]
  functions = {
    "create": create,
  }
  output = functions[command](argument_words, argument_file)
  print(yaml.dump({"output": output}))
