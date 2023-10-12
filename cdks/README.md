### Context Driven Knowledge Protocols

The advent of Large Language Models (LLMs) has been fallen short from impressive. These AI models can achieve impressive feats
on text generation, summarization, structuring data, debug and write code and also (more basic) problem solving skills. Current state
of the art LLMs have been trained on large textual corpus (potentially close to all internet content) as well as fine tuned and later
improved using advanced techniques such as Reinforcement Learning from Human Feedback (RLHF). It is important to note, however, that
the majority of the LLMs capacities are direct consequence of the fact that many problems and contexts can be naturally expressed in
natural language. 

From empirical experience, one notices that LLMs are particularly good at generating structured data, as opposed to unstructured data. For
example, even though state of the art LLMs text generation capacities are impressive, these models tend to condense important information
and usually they are not capable to generate large quantities of technical expertise textual corpus.

Moreover, LLMs are not necessarily knowledgeable artifacts. Instead, we proclaim that LLMs are akin to generators (in usual programming 
language terms), while following extremely complex probabilitic densifty functions (PDFs). This analogy shows that LLMs are objects
that, by its very nature, are difficult to protocolize, that is, to build protocols on top of it, like decentralized networks. This
issue can be, possibly, minimized by fine tuned over different sets of specific tasks. However, it is difficult to coordinate multiple
LLMs operating on very distinct tasks. 

Moreover, LLMs, due to its costs and hardware requirements, tend to be orchestrated by very centralized entities. This brings a major
restriction to the adoption of these technologies to automatize a larger number of applications of society sectors. Indeed, it is
expected that large institutions will not be happy to share their internal data with OpenAI, or other big tech companies.

For this reason, we propose a different methodology, that of a `Context Driven Knowledge` protocols (CDKs). LLMs habe proved to be
suitable for problems akin to transform and extract contents from text into a more structured representations. The latter can be either
in the form of Knowledge Graphs (KGs), Code programs (see Voyager paper), or even Named Entity Relations, etc. The later form
data structures (very different in nature to usual generators). The main advantage of these later representations is that these can 
be more easily coordinated across networks and they satisfy a `locality` condition. That is, in order to have acess to specific chunks
of knowledge, we don't need to deal with terabytes of global parameters (like in LLMs).

Moreover, such structures are suitable for LLM interaction. Indeed, they provide natural frameworks for fine tuning of LLMs, etc. 

### Design

We propose a simple framework to protocolize a network of Knowledge Graphs, boosted by the powers of LLMs.
