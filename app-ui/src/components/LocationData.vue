<template>
  <div v-if="location">
    <div v-if="!isEdit">
      <h1 class="title">
        {{ location.name }}
        <button class="button mr-1" v-on:click="isEdit = true">Edit</button>
        <button class="button is-danger" v-on:click="deleteLocation">
          Delete
        </button>
      </h1>
      <p><strong>ID</strong>: {{ location.id }}</p>
      <p><strong>Address</strong>: {{ location.address ?? '--' }}</p>
      <hr />
      <h1 class="subtitle">Inventory</h1>
      <div class="table-container" v-if="location.transactions.length > 0">
        <table class="table">
          <thead>
            <tr>
              <th>Item ID</th>
              <th>Item SKU</th>
              <th>Item Name</th>
              <th>Quantity</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in inventory" :key="item[0]">
              <td>{{ item[0] }}</td>
              <td>{{ item[1].sku ?? '--' }}</td>
              <td>{{ item[1].name ?? '--' }}</td>
              <td>{{ item[1].quantity }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div v-else>
        <p>No inventory at location.</p>
      </div>
      <hr />
      <h1 class="subtitle">Transactions</h1>
      <div class="table-container" v-if="location.transactions.length > 0">
        <table class="table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Item ID</th>
              <th>Item SKU</th>
              <th>Item Name</th>
              <th>Transaction Date</th>
              <th>Quantity</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="transaction in location.transactions"
              :key="transaction.id"
            >
              <td>{{ transaction.id }}</td>
              <td>{{ transaction.item.id ?? '--' }}</td>
              <td>{{ transaction.item.sku ?? '--' }}</td>
              <td>{{ transaction.item.name ?? '--' }}</td>
              <td>{{ transaction.transactionDate ?? '--' }}</td>
              <td>{{ transaction.quantity }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div v-else>
        <p>No transactions for location.</p>
      </div>
    </div>
    <div v-else>
      <location-change
        v-on:submit="isEdit = false"
        :initialLocationName="location.name"
        :initialAddress="location.address"
      />
    </div>
  </div>
  <div v-else>
    <p>Location not found.</p>
  </div>
</template>
<script>
import { useRoute } from 'vue-router';
import { useClientHandle } from '@urql/vue';
import LocationChange from '../components/LocationChange.vue';

export default {
  data: () => {
    return {
      isEdit: false,
    };
  },
  components: {
    LocationChange,
  },
  async setup() {
    const handle = useClientHandle();
    const route = useRoute();
    let id = route.params.id;

    // get the initial location
    let result = await handle
      .useQuery({
        query: `
      {
        location(id: ${id}) {
          id,
          name,
          address,
          transactions {
            id,
            item {
              id,
              sku,
              name,
            },
            transactionDate,
            quantity,
          },
        }
      }
      `,
      })
      .executeQuery();
    if (result.data?.value?.location) {
      result = JSON.parse(JSON.stringify(result.data?.value?.location));
      // update the location live
      const handleSubscription = (_, response) => {
        const event = response.locationSubscription;
        if (event.data.id == id) {
          if (event.modification === 'UPDATE') {
            return event.data;
          }
        }
      };

      // subscribe to location changes
      // note this is very inefficient
      const subscription = handle.useSubscription(
        {
          query: `
        subscription {
          locationSubscription {
            modification,
            data {
                id,
                name,
                address,
                transactions {
                    id,
                    item {
                    id,
                    sku,
                    name,
                    },
                    transactionDate,
                    quantity,
                },
            }
          }
        }
        `,
        },
        handleSubscription
      );

      const deleteLocation = handle.useMutation(`
        mutation ($id: LocationId!) {
          deleteLocation(id: $id) {
            id
          }
        }
      `);

      // hack to get things working
      return {
        base: result,
        live: subscription.data,
        deleteLocation() {
          deleteLocation
            .executeMutation({ id: parseInt(id) })
            .then((result) => {
              if (result.error) {
                alert(JSON.stringify(result.error.graphQLErrors[0]));
              } else {
                alert('Deleted!');
                this.$router.push('/locations');
              }
            });
        },
      };
    } else {
      return {
        base: undefined,
        live: undefined,
      };
    }
  },
  computed: {
    location: function () {
      return this.live ?? this.base;
    },
    inventory: function () {
      const transactions = this.location.transactions;
      let items = new Map();
      for (const transaction of transactions) {
        if (items.has(transaction.item.id)) {
          items.get(transaction.item.id).quantity += transaction.quantity;
        } else {
          items.set(transaction.item.id, {
            name: transaction.item.name,
            sku: transaction.item.sku,
            quantity: transaction.quantity,
          });
        }
      }
      console.log(transactions);
      return Array.from(items);
    },
  },
};
</script>