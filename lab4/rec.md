```bash
tern@ubuntu:~/Documents/codes/python-codes/trial$ conda init bash
no change     /home/tern/miniconda3/condabin/conda
no change     /home/tern/miniconda3/bin/conda
no change     /home/tern/miniconda3/bin/conda-env
no change     /home/tern/miniconda3/bin/activate
no change     /home/tern/miniconda3/bin/deactivate
no change     /home/tern/miniconda3/etc/profile.d/conda.sh
no change     /home/tern/miniconda3/etc/fish/conf.d/conda.fish
no change     /home/tern/miniconda3/shell/condabin/Conda.psm1
no change     /home/tern/miniconda3/shell/condabin/conda-hook.ps1
no change     /home/tern/miniconda3/lib/python3.10/site-packages/xontrib/conda.xsh
no change     /home/tern/miniconda3/etc/profile.d/conda.csh
modified      /home/tern/.bashrc

==> For changes to take effect, close and re-open your current shell. <==

tern@ubuntu:~/Documents/codes/python-codes/trial$ source ~/.bashrc
(base) tern@ubuntu:~/Documents/codes/python-codes/trial$ conda activate ray
(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ ray start --head
Usage stats collection is enabled. To disable this, add `--disable-usage-stats` to the command that starts the cluster, or run the following command: `ray disable-usage-stats` before starting the cluster. See https://docs.ray.io/en/master/cluster/usage-stats.html for more details.

Local node IP: 114.214.252.223

--------------------
Ray runtime started.
--------------------

Next steps
  To connect to this Ray runtime from another node, run
    ray start --address='114.214.252.223:6379'
  
  Alternatively, use the following Python code:
    import ray
    ray.init(address='auto')
  
  To connect to this Ray runtime from outside of the cluster, for example to
  connect to a remote cluster from your laptop directly, use the following
  Python code:
    import ray
    ray.init(address='ray://<head_node_ip_address>:10001')
  
  If connection fails, check your firewall settings and network configuration.
  
  To terminate the Ray runtime, run
    ray stop
(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ python markov_new.py 100000
2023-06-20 11:29:11,978	INFO worker.py:1342 -- Connecting to existing Ray cluster at address: 114.214.252.223:6379...
2023-06-20 11:29:11,982	INFO worker.py:1525 -- Connected to Ray cluster. View the dashboard at http://127.0.0.1:8265 
2023-06-20 11:29:12,324	WARNING worker.py:1839 -- Warning: More than 5000 tasks are pending submission to actor 357597ca967e7d7d633102dd01000000. To reduce memory usage, wait for these tasks to finish before sending more.
2023-06-20 11:29:12,390	WARNING worker.py:1839 -- Warning: More than 10000 tasks are pending submission to actor 357597ca967e7d7d633102dd01000000. To reduce memory usage, wait for these tasks to finish before sending more.
2023-06-20 11:29:12,661	WARNING worker.py:1839 -- Warning: More than 20000 tasks are pending submission to actor 357597ca967e7d7d633102dd01000000. To reduce memory usage, wait for these tasks to finish before sending more.
2023-06-20 11:29:13,157	WARNING worker.py:1839 -- Warning: More than 40000 tasks are pending submission to actor 357597ca967e7d7d633102dd01000000. To reduce memory usage, wait for these tasks to finish before sending more.
2023-06-20 11:29:14,211	WARNING worker.py:1839 -- Warning: More than 80000 tasks are pending submission to actor 357597ca967e7d7d633102dd01000000. To reduce memory usage, wait for these tasks to finish before sending more.
markov_new.py:18: RuntimeWarning: overflow encountered in matmul
  return np.matmul(matrix1, matrix2)
total duration:  27.68019461631775
(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ python markov_new.py 10000
2023-06-20 11:29:56,011	INFO worker.py:1342 -- Connecting to existing Ray cluster at address: 114.214.252.223:6379...
2023-06-20 11:29:56,015	INFO worker.py:1525 -- Connected to Ray cluster. View the dashboard at http://127.0.0.1:8265 
2023-06-20 11:29:56,354	WARNING worker.py:1839 -- Warning: More than 5000 tasks are pending submission to actor 13f1dd6bf15bf18c54a4e4c402000000. To reduce memory usage, wait for these tasks to finish before sending more.
markov_new.py:18: RuntimeWarning: overflow encountered in matmul
  return np.matmul(matrix1, matrix2)
total duration:  11.307637929916382
(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ python markov_new.py 10
2023-06-20 11:30:27,501	INFO worker.py:1342 -- Connecting to existing Ray cluster at address: 114.214.252.223:6379...
2023-06-20 11:30:27,506	INFO worker.py:1525 -- Connected to Ray cluster. View the dashboard at http://127.0.0.1:8265 
total duration:  9.69461703300476
(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ python markov_new.py 1
2023-06-20 11:30:51,152	INFO worker.py:1342 -- Connecting to existing Ray cluster at address: 114.214.252.223:6379...
2023-06-20 11:30:51,157	INFO worker.py:1525 -- Connected to Ray cluster. View the dashboard at http://127.0.0.1:8265 
total duration:  9.28380298614502
```

