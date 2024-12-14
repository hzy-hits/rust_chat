<template>
  <div class="flex-1 overflow-y-auto p-5 mb-10" ref="messageContainer">
    <div v-if="messages.length === 0" class="text-center text-gray-400 mt-5">
      No messages in this channel yet.
    </div>
    <div v-else>
      <div
        v-for="message in messages"
        :key="message.id"
        class="flex items-start mb-5"
      >
        <img
          :src="`https://ui-avatars.com/api/?name=${getUsername(message.senderId)}`"
          class="w-10 h-10 rounded-full mr-3"
          alt="Avatar"
        />
        <div class="max-w-4/5">
          <div class="flex items-center mb-1">
            <span class="font-bold mr-2">{{ getUsername(message.senderId) }}</span>
            <span class="text-xs text-gray-500">{{
              message.formattedCreatedAt || message.createdAt
            }}</span>
          </div>
          <div class="text-sm leading-relaxed break-words whitespace-pre-wrap">
            {{ message.content }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  computed: {
    messages() {
      return this.$store.getters.getMessagesForActiveChannel;
    },
    activeChannelId() {
      let channel = this.$store.state.activeChannel;
      return channel ? channel.id : null;
    },
    // 添加用户数据就绪状态的计算属性
    usersLoaded() {
      return Object.keys(this.$store.state.users || {}).length > 0;
    }
  },
  watch: {
    messages: {
      handler() {
        this.$nextTick(() => {
          this.scrollToBottom();
        });
      },
      deep: true,
    },
    activeChannelId(newChannelId) {
      if (newChannelId) {
        this.fetchMessages(newChannelId);
      }
    },
  },
  methods: {
    fetchMessages(channelId) {
      this.$store.dispatch("fetchMessagesForChannel", channelId);
    },
    getSender(userId) {
      return this.$store.getters.getUserById(userId) || { username: 'Unknown User' };
    },
    // 新增一个专门处理用户名的方法
    getUsername(userId) {
      const user = this.getSender(userId);
      return (user?.username || 'Unknown User').replace(' ', '+');
    },
    scrollToBottom() {
      const container = this.$refs.messageContainer;
      if (container) {
        container.scrollTop = container.scrollHeight;
      }
    },
  },
  async mounted() {
    // 确保用户数据已加载
    if (!this.usersLoaded) {
      await this.$store.dispatch('fetchWorkspaceUsers');
    }
    if (this.activeChannelId) {
      await this.fetchMessages(this.activeChannelId);
    }
    this.scrollToBottom();
  },
  updated() {
    this.scrollToBottom();
  },
};
</script>
