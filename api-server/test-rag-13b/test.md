# Issue of api-server with llama-2-13b

## Introduction

在使用 [llama-2-3b](https://huggingface.co/second-state/Llama-2-13B-Chat-GGUF/blob/main/Llama-2-13b-chat-hf-Q5_K_M.gguf) 启动 api-server，然后进行 RAG 测试。这个测试包含了两个步骤：

- 第一步发送 document chunks，计算 embeddings，并将 embeddings 保存到数据库。
- 第二步发送 user query，计算 query embedding；然后，从数据库中检索最相似的 document chunks，构成context；将 user query text 和 context 组成 messages，完成chat completion，返回结果。

问题出现在第二步。在 ctx-size 设置为5120 (llama-2-13b模型的最大context size)，在第二步的chat completion阶段，返回的字符串（bytes to string）是 embeddings 信息，而不是chat信息。在其他设置不变的情况下，将 ctx-size 设置为512，返回的字符串是正确的chat信息。两种设置的结果，分别存放在 `5120.txt` 和 `4096.txt`。

## Steps to reproduce

- 启动 api-server

  ```bash
  wasmedge --dir .:. --nn-preload default:GGML:AUTO:Llama-2-13b-chat-hf-Q5_K_M.gguf llama-api-server.wasm \
  --prompt-template llama-2-chat \
  --ctx-size 5120 \
  --qdrant-url http://127.0.0.1:6333 \
  --qdrant-collection-name "paris" \
  --qdrant-limit 3 --log-prompts
  ```

- 启动 Qdrant

  ```bash
  # Pull the Qdrant docker image
  docker pull qdrant/qdrant

  # Create a directory to store Qdrant data
  mkdir qdrant_storage

  # Run Qdrant service
  docker run -p 6333:6333 -p 6334:6334 -v $(pwd)/qdrant_storage:/qdrant/storage:z qdrant/qdrant
  ```

- 运行测试

  运行过程分为两步。每一步的结果，可以在api-server的terminal中看到。

  - 第一步: 发送 document chunks

    ```bash
    curl -s -X POST http://localhost:8080/v1/embeddings \
    -H 'accept:application/json' \
    -H 'Content-Type: application/json' \
    -d '{"model":"dummy-embedding-model","input":["Paris, city and capital of France, situated in the north-central part of the country. People were living on the site of the present-day city, located along the Seine River some 233 miles (375 km) upstream from the river’s mouth on the English Channel (La Manche), by about 7600 BCE. The modern city has spread from the island (the Île de la Cité) and far beyond both banks of the Seine.","Paris occupies a central position in the rich agricultural region known as the Paris Basin, and it constitutes one of eight départements of the Île-de-France administrative region. It is by far the country’s most important centre of commerce and culture. Area city, 41 square miles (105 square km); metropolitan area, 890 square miles (2,300 square km).","Pop. (2020 est.) city, 2,145,906; (2020 est.) urban agglomeration, 10,858,874.","For centuries Paris has been one of the world’s most important and attractive cities. It is appreciated for the opportunities it offers for business and commerce, for study, for culture, and for entertainment; its gastronomy, haute couture, painting, literature, and intellectual community especially enjoy an enviable reputation. Its sobriquet “the City of Light” (“la Ville Lumière”), earned during the Enlightenment, remains appropriate, for Paris has retained its importance as a centre for education and intellectual pursuits.","Paris’s site at a crossroads of both water and land routes significant not only to France but also to Europe has had a continuing influence on its growth. Under Roman administration, in the 1st century BCE, the original site on the Île de la Cité was designated the capital of the Parisii tribe and territory. The Frankish king Clovis I had taken Paris from the Gauls by 494 CE and later made his capital there.","Under Hugh Capet (ruled 987–996) and the Capetian dynasty the preeminence of Paris was firmly established, and Paris became the political and cultural hub as modern France took shape. France has long been a highly centralized country, and Paris has come to be identified with a powerful central state, drawing to itself much of the talent and vitality of the provinces."]}'
    ```

  - 第二步: 发送 user query

    ```bash
    curl -s -X POST http://localhost:8080/v1/chat/completions \
        -H 'accept:application/json' \
        -H 'Content-Type: application/json' \
        -d '{"messages":[{"role":"user","content":"What is the location of Paris, France on the Seine River?\n"}],"model":"llama-2-7b","stream":false}'
    ```
